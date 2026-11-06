//! Reads and verified writes of the mouse settings.

use crate::device::{
    AngleValue, Battery, DevicePath, Inbox, Link, Model, MouseError, MouseSnapshot, Session,
};
use crate::protocol::{Command, Frame, ProtocolError, address, check};
use crate::settings::{
    AngleBlock, Button, ButtonAction, DpiStage, FirmwareVersion, GeneralBlock, LiftOff,
    PollingRate, SensorBlock, Shortcut, SleepBlock,
};

const STAGE_LEN: usize = 4;
const PAIR_LEN: usize = 8;
const PAIR_COUNT: usize = 4;
const MAX_READ: u8 = 10;
const SECONDS_PER_SLEEP_BYTE: u16 = 10;

/// A connected, identified mouse.
#[derive(Debug)]
pub struct Mouse<L, C> {
    session: Session<L, C>,
    model: &'static Model,
    path: DevicePath,
}

impl<L: Link, C> Mouse<L, C> {
    /// Checks that the mouse answers (dongle path only) and reads its identity.
    ///
    /// # Errors
    /// [`MouseError::NotConnected`] when the dongle reports no linked mouse,
    /// [`MouseError::UnsupportedModel`] for an unknown identity, and any exchange error.
    pub fn identify(
        session: &mut Session<L, C>,
        path: DevicePath,
    ) -> Result<&'static Model, MouseError> {
        if path == DevicePath::Dongle && !Self::is_online(session)? {
            return Err(MouseError::NotConnected);
        }

        let identity = session.exchange(&Frame::request(Command::GetMouseCidMid))?;
        let (cid, mid) = (identity.payload()[0], identity.payload()[1]);

        Model::find(cid, mid).ok_or(MouseError::UnsupportedModel { cid, mid })
    }

    /// Asks the dongle whether the mouse is linked to it. The dongle answers this itself,
    /// also with the mouse off, so the question costs the mouse nothing.
    ///
    /// # Errors
    /// Any exchange error.
    pub(crate) fn is_online(session: &mut Session<L, C>) -> Result<bool, MouseError> {
        let online = session.exchange(&Frame::request(Command::GetWirelessMouseOnline))?;

        Ok(online.payload()[0] == 1)
    }

    /// Wraps an identified session.
    #[must_use]
    pub fn new(session: Session<L, C>, model: &'static Model, path: DevicePath) -> Self {
        Self {
            session,
            model,
            path,
        }
    }

    /// Gives back the inbox with its deferred inputs.
    #[must_use]
    pub fn into_inbox(self) -> Inbox<C> {
        self.session.into_inbox()
    }

    /// Gives access to the session, for the thread that waits on its inbox.
    pub(crate) fn session_mut(&mut self) -> &mut Session<L, C> {
        &mut self.session
    }

    /// Gives back the session with its open link.
    #[must_use]
    pub(crate) fn into_session(self) -> Session<L, C> {
        self.session
    }

    /// Reads every setting from the mouse.
    ///
    /// # Errors
    /// Any exchange error, and a protocol error for a malformed block, a bad check on a
    /// used DPI stage, a button or a shortcut. A bad check on an unused stage is ignored.
    /// A sensor block, firmware or far distance read the mouse rejects, and a sensor value
    /// that cannot be read, leave that field `None`.
    pub fn snapshot(&mut self) -> Result<MouseSnapshot, MouseError> {
        let profile = self
            .session
            .exchange(&Frame::request(Command::GetCurrentConfig))?;
        let profile = profile.payload()[0];
        let battery = self.battery()?;
        let block = self.read_block()?;

        let sensor = self.model.sensor;
        let raw_stages = self.read_stages()?;
        let stages = (0..usize::from(block.stage_count()))
            .map(|index| sensor.decode_stage(stage_bytes(&raw_stages, index)))
            .collect::<Result<Vec<_>, _>>()?;

        let buttons = self.read_buttons()?;
        let shortcuts = self.read_shortcuts(&buttons)?;

        let firmware = rejected_as_none(self.read_firmware())?;
        let far_distance = rejected_as_none(self.read_far_distance())?.and_then(check::flag);
        let lift_off = rejected_as_none(self.read_lift_off())?;
        let sensor_block = rejected_as_none(self.read_sensor_block())?;
        let angle = self.read_angle(firmware)?;

        Ok(MouseSnapshot {
            model_name: self.model.name.to_owned(),
            cid: self.model.cid,
            mid: self.model.mid,
            profile,
            battery,
            polling: block.polling(),
            polling_rates: self.model.polling_rates(self.path).to_vec(),
            sensor,
            current_stage: block.current_stage(),
            stages,
            buttons,
            shortcuts,
            firmware,
            lift_off_tenths: lift_off.and_then(|item| item.millimetres_tenths()),
            debounce_ms: sensor_block.and_then(|block| block.debounce_ms()),
            debounce_options_ms: self.model.debounce_ms.to_vec(),
            motion_sync: sensor_block.and_then(|block| block.motion_sync()),
            linear_correction: sensor_block.and_then(|block| block.linear_correction()),
            ripple_control: sensor_block.and_then(|block| block.ripple_control()),
            sleep_seconds: sensor_block
                .and_then(|block| block.led_off_byte())
                .map(|byte| u16::from(byte) * SECONDS_PER_SLEEP_BYTE),
            sleep_options_seconds: self.model.sleep_seconds.to_vec(),
            angle,
            far_distance,
        })
    }

    /// Reads the battery state.
    ///
    /// # Errors
    /// Any exchange error, and `InvalidValue` for a percentage above 100.
    pub fn battery(&mut self) -> Result<Battery, MouseError> {
        let reply = self
            .session
            .exchange(&Frame::request(Command::GetBatteryLevel))?;
        let percent = reply.payload()[0];

        if percent > 100 {
            return Err(ProtocolError::InvalidValue("battery percent").into());
        }

        Ok(Battery {
            percent,
            charging: reply.payload()[1] != 0,
        })
    }

    /// Sets the polling rate, keeping the other bytes of the general block.
    ///
    /// # Errors
    /// `InvalidValue` when the path does not offer `rate`, and any read or write error.
    pub fn set_polling(&mut self, rate: PollingRate) -> Result<(), MouseError> {
        if !self.model.polling_rates(self.path).contains(&rate) {
            return Err(ProtocolError::InvalidValue("polling rate for this connection").into());
        }

        let block = self.read_block()?;

        self.write_block(&block.with_polling(rate))
    }

    /// Selects the current DPI stage.
    ///
    /// # Errors
    /// `InvalidValue` when `index` is not below the stage count, and any read or write error.
    pub fn set_current_stage(&mut self, index: u8) -> Result<(), MouseError> {
        let block = self.read_block()?;

        self.write_block(&block.with_current_stage(index)?)
    }

    /// Changes the number of DPI stages. Growing first gives every new slot that does not
    /// decode a copy of the last used stage, then writes the block.
    ///
    /// # Errors
    /// `InvalidValue` for a count outside 1..=8, a bad check on the last used stage, and
    /// any read or write error.
    pub fn set_stage_count(&mut self, count: u8) -> Result<(), MouseError> {
        let block = self.read_block()?;
        let old_count = usize::from(block.stage_count());

        let mut resized = block.with_stage_count(count)?;
        if block.current_stage() >= count {
            resized = block
                .with_current_stage(count - 1)?
                .with_stage_count(count)?;
        }

        if usize::from(count) > old_count {
            self.fill_new_stages(old_count, usize::from(count))?;
        }

        self.write_block(&resized)
    }

    /// Sets the DPI of one stage, keeping the other stage of its pair.
    ///
    /// # Errors
    /// `InvalidValue` for an off-grid value or an index not below the stage count, and any
    /// read or write error.
    pub fn set_stage_dpi(&mut self, index: u8, stage: DpiStage) -> Result<(), MouseError> {
        let encoded = self.model.sensor.encode_stage(stage)?;
        let block = self.read_block()?;

        if index >= block.stage_count() {
            return Err(ProtocolError::InvalidValue("stage index").into());
        }

        let address = pair_address(usize::from(index) / 2);
        let mut pair = self.read_pair(address)?;
        let offset = usize::from(index % 2) * STAGE_LEN;
        pair[offset..offset + STAGE_LEN].copy_from_slice(&encoded);

        self.write_eeprom(address, &pair)
    }

    /// Assigns an action to a button. Shortcuts go through [`Mouse::set_shortcut`].
    ///
    /// # Errors
    /// `InvalidValue` for `Other` and `Shortcut`, or when the button's current action is
    /// not editable; `BadCheck` when the current bytes are corrupt; any read or write error.
    pub fn set_button(&mut self, button: Button, action: ButtonAction) -> Result<(), MouseError> {
        if matches!(action, ButtonAction::Shortcut) {
            return Err(ProtocolError::InvalidValue("shortcut needs set_shortcut").into());
        }

        let encoded = encode_editable(action)?;
        self.require_editable(button)?;

        self.write_eeprom(button.address(), &encoded)
    }

    /// Stores a shortcut for a button, then switches the button to it.
    ///
    /// # Errors
    /// `InvalidValue` when the button's current action is not editable; `BadCheck` when
    /// its current bytes are corrupt (both before any write); any read or write error.
    pub fn set_shortcut(&mut self, button: Button, shortcut: &Shortcut) -> Result<(), MouseError> {
        self.require_editable(button)?;

        for (index, frame) in shortcut.to_frames().iter().enumerate() {
            self.write_eeprom(button.shortcut_address() + 10 * index as u16, frame)?;
        }

        let action = encode_editable(ButtonAction::Shortcut)?;

        self.write_eeprom(button.address(), &action)
    }

    /// Sets the lift-off height, in tenths of a millimetre.
    ///
    /// # Errors
    /// `InvalidValue` for a height outside 7..=17 or a stored height that cannot be read, and
    /// any read or write error.
    pub fn set_lift_off(&mut self, tenths: u8) -> Result<(), MouseError> {
        let item = self.read_lift_off()?.with_tenths(tenths)?;

        self.write_eeprom(address::LIFT_OFF, item.as_bytes())
    }

    /// Sets the debounce time, keeping the other pairs of the sensor block.
    ///
    /// # Errors
    /// `InvalidValue` for a time the model does not offer, `BadCheck` when the stored time
    /// is corrupt, and any read or write error.
    pub fn set_debounce(&mut self, ms: u8) -> Result<(), MouseError> {
        if !self.model.debounce_ms.contains(&ms) {
            return Err(ProtocolError::InvalidValue("debounce time").into());
        }

        let block = self.read_sensor_block()?.with_debounce_ms(ms)?;

        self.write_eeprom(address::SENSOR_BLOCK, block.as_bytes())
    }

    /// Turns motion sync on or off, keeping the other pairs of the sensor block.
    ///
    /// # Errors
    /// `BadCheck` or `InvalidValue` when the stored value cannot be read, and any read or
    /// write error.
    pub fn set_motion_sync(&mut self, on: bool) -> Result<(), MouseError> {
        let block = self.read_sensor_block()?.with_motion_sync(on)?;

        self.write_eeprom(address::SENSOR_BLOCK, block.as_bytes())
    }

    /// Turns linear correction on or off, keeping the other pairs of the sensor block.
    ///
    /// # Errors
    /// `BadCheck` or `InvalidValue` when the stored value cannot be read, and any read or
    /// write error.
    pub fn set_linear_correction(&mut self, on: bool) -> Result<(), MouseError> {
        let block = self.read_sensor_block()?.with_linear_correction(on)?;

        self.write_eeprom(address::SENSOR_BLOCK, block.as_bytes())
    }

    /// Turns ripple control on or off, keeping the other pairs of the sensor block.
    ///
    /// # Errors
    /// `BadCheck` or `InvalidValue` when the stored value cannot be read, and any read or
    /// write error.
    pub fn set_ripple_control(&mut self, on: bool) -> Result<(), MouseError> {
        let block = self.read_sensor_block()?.with_ripple_control(on)?;

        self.write_eeprom(address::SENSOR_BLOCK, block.as_bytes())
    }

    /// Sets the sleep time as ATK HUB does: the LED off time in the sensor block first, then
    /// the sensor sleep time with its enabled flag. Both blocks are read and checked before
    /// the first write.
    ///
    /// When the second write fails the LED off time stays changed, and a snapshot shows only
    /// that half. Setting the same time again writes both blocks and heals it.
    ///
    /// # Errors
    /// `InvalidValue` for a time the model does not offer, `BadCheck` or `InvalidValue` when
    /// a stored value of these three cannot be read, and any read or write error.
    pub fn set_sleep(&mut self, seconds: u16) -> Result<(), MouseError> {
        if !self.model.sleep_seconds.contains(&seconds) {
            return Err(ProtocolError::InvalidValue("sleep time").into());
        }

        let byte = u8::try_from(seconds / SECONDS_PER_SLEEP_BYTE)
            .map_err(|_| ProtocolError::InvalidValue("sleep time"))?;
        let sensor_block = self.read_sensor_block()?.with_led_off_byte(byte)?;
        let sleep_block = self.read_sleep_block()?.with_sleep(byte)?;

        self.write_eeprom(address::SENSOR_BLOCK, sensor_block.as_bytes())?;

        self.write_eeprom(address::SLEEP_BLOCK, sleep_block.as_bytes())
    }

    /// Sets the sensor angle in degrees, keeping the switch.
    ///
    /// # Errors
    /// `InvalidValue` for an angle outside -30..=30 or a firmware without the angle,
    /// `BadCheck` when the stored angle is corrupt, and any read or write error.
    pub fn set_angle(&mut self, degrees: i8) -> Result<(), MouseError> {
        let block = self.read_angle_block_to_change()?.with_angle(degrees)?;

        self.write_eeprom(address::ANGLE_BLOCK, block.as_bytes())
    }

    /// Turns the sensor angle on or off, keeping the angle.
    ///
    /// # Errors
    /// `InvalidValue` for a firmware without the angle, `BadCheck` or `InvalidValue` when the
    /// stored switch cannot be read, and any read or write error.
    pub fn set_angle_enabled(&mut self, on: bool) -> Result<(), MouseError> {
        let block = self.read_angle_block_to_change()?.with_enabled(on)?;

        self.write_eeprom(address::ANGLE_BLOCK, block.as_bytes())
    }

    /// Turns far distance mode on or off, then reads it back: it is no EEPROM item.
    ///
    /// # Errors
    /// `InvalidValue` when the stored mode is neither 0 nor 1, any exchange error, with
    /// nothing sent after a failed read, and [`MouseError::FarDistanceMismatch`] when the
    /// mouse holds another mode afterwards.
    pub fn set_far_distance(&mut self, enabled: bool) -> Result<(), MouseError> {
        if check::flag(self.read_far_distance()?).is_none() {
            return Err(ProtocolError::InvalidValue("far distance mode").into());
        }

        self.session.exchange(&Frame::set_far_distance(enabled))?;

        let written = u8::from(enabled);
        let actual = self.read_far_distance()?;

        if actual != written {
            return Err(MouseError::FarDistanceMismatch { written, actual });
        }

        Ok(())
    }

    /// Reads `len` EEPROM bytes at `address`.
    ///
    /// # Errors
    /// `BadLength` for `len` above 10, and any exchange error.
    pub fn read_eeprom(&mut self, address: u16, len: u8) -> Result<Vec<u8>, MouseError> {
        let reply = self.session.exchange(&Frame::get_eeprom(address, len)?)?;

        Ok(reply.payload()[..usize::from(len)].to_vec())
    }

    /// Writes EEPROM bytes, then reads them back and compares.
    ///
    /// # Errors
    /// Any exchange error, with no read-back after a failed write, and
    /// [`MouseError::ReadBackMismatch`] when the mouse holds other bytes afterwards.
    pub fn write_eeprom(&mut self, address: u16, data: &[u8]) -> Result<(), MouseError> {
        self.session.exchange(&Frame::set_eeprom(address, data)?)?;

        let len = u8::try_from(data.len()).expect("a frame holds at most 10 bytes");
        let actual = self.read_eeprom(address, len)?;

        if actual != data {
            return Err(MouseError::ReadBackMismatch {
                address,
                written: data.to_vec(),
                actual,
            });
        }

        Ok(())
    }

    fn read_block(&mut self) -> Result<GeneralBlock, MouseError> {
        let bytes = self.read_eeprom(address::GENERAL_BLOCK, MAX_READ)?;
        let raw = bytes.try_into().map_err(|_| ProtocolError::BadLength)?;

        Ok(GeneralBlock::from_raw(raw)?)
    }

    fn write_block(&mut self, block: &GeneralBlock) -> Result<(), MouseError> {
        self.write_eeprom(address::GENERAL_BLOCK, block.as_bytes())
    }

    fn read_array<const N: usize>(&mut self, address: u16) -> Result<[u8; N], MouseError> {
        let bytes = self.read_eeprom(address, N as u8)?;

        Ok(bytes.try_into().map_err(|_| ProtocolError::BadLength)?)
    }

    fn read_pair(&mut self, address: u16) -> Result<[u8; PAIR_LEN], MouseError> {
        self.read_array(address)
    }

    fn read_lift_off(&mut self) -> Result<LiftOff, MouseError> {
        Ok(LiftOff::from_raw(self.read_array(address::LIFT_OFF)?))
    }

    fn read_sensor_block(&mut self) -> Result<SensorBlock, MouseError> {
        Ok(SensorBlock::from_raw(
            self.read_array(address::SENSOR_BLOCK)?,
        ))
    }

    fn read_sleep_block(&mut self) -> Result<SleepBlock, MouseError> {
        Ok(SleepBlock::from_raw(self.read_array(address::SLEEP_BLOCK)?))
    }

    fn read_angle_block(&mut self) -> Result<AngleBlock, MouseError> {
        Ok(AngleBlock::from_raw(self.read_array(address::ANGLE_BLOCK)?))
    }

    /// Reads the angle for a snapshot, with no exchange when `firmware` has no angle.
    fn read_angle(
        &mut self,
        firmware: Option<FirmwareVersion>,
    ) -> Result<Option<AngleValue>, MouseError> {
        if !self.model.supports_angle(firmware) {
            return Ok(None);
        }

        let block = rejected_as_none(self.read_angle_block())?;

        Ok(block.map(|block| AngleValue {
            degrees: block.angle(),
            enabled: block.enabled(),
        }))
    }

    /// Asks the mouse for its firmware and reads the angle block when it offers the angle.
    fn read_angle_block_to_change(&mut self) -> Result<AngleBlock, MouseError> {
        let firmware = rejected_as_none(self.read_firmware())?;

        if !self.model.supports_angle(firmware) {
            return Err(ProtocolError::InvalidValue("angle on this firmware").into());
        }

        self.read_angle_block()
    }

    fn read_firmware(&mut self) -> Result<FirmwareVersion, MouseError> {
        let reply = self
            .session
            .exchange(&Frame::request(Command::GetMouseVersion))?;

        Ok(FirmwareVersion {
            major: reply.payload()[0],
            minor: reply.payload()[1],
        })
    }

    fn read_far_distance(&mut self) -> Result<u8, MouseError> {
        let reply = self
            .session
            .exchange(&Frame::request(Command::GetFarDistanceMode))?;

        Ok(reply.payload()[0])
    }

    fn read_stages(&mut self) -> Result<[u8; PAIR_LEN * PAIR_COUNT], MouseError> {
        let mut raw = [0; PAIR_LEN * PAIR_COUNT];

        for pair in 0..PAIR_COUNT {
            raw[pair * PAIR_LEN..(pair + 1) * PAIR_LEN]
                .copy_from_slice(&self.read_pair(pair_address(pair))?);
        }

        Ok(raw)
    }

    /// Writes a copy of the last used stage into every new slot that does not decode.
    fn fill_new_stages(&mut self, old_count: usize, new_count: usize) -> Result<(), MouseError> {
        let sensor = self.model.sensor;
        let mut raw = self.read_stages()?;
        let last = stage_bytes(&raw, old_count - 1);
        sensor.decode_stage(last)?;

        let mut changed = [false; PAIR_COUNT];
        for index in old_count..new_count {
            if sensor.decode_stage(stage_bytes(&raw, index)).is_err() {
                raw[index * STAGE_LEN..(index + 1) * STAGE_LEN].copy_from_slice(&last);
                changed[index / 2] = true;
            }
        }

        for pair in (0..PAIR_COUNT).filter(|&pair| changed[pair]) {
            self.write_eeprom(
                pair_address(pair),
                &raw[pair * PAIR_LEN..(pair + 1) * PAIR_LEN],
            )?;
        }

        Ok(())
    }

    fn read_buttons(&mut self) -> Result<Vec<(Button, ButtonAction)>, MouseError> {
        let mut table = self.read_eeprom(address::BUTTONS, MAX_READ)?;
        table.extend(self.read_eeprom(address::BUTTONS + u16::from(MAX_READ), MAX_READ)?);

        self.model
            .buttons
            .iter()
            .map(|&button| {
                let start = usize::from(button.address() - address::BUTTONS);
                let bytes = table[start..start + 4].try_into().expect("four bytes");

                Ok((button, ButtonAction::decode(bytes)?))
            })
            .collect()
    }

    fn read_shortcuts(
        &mut self,
        buttons: &[(Button, ButtonAction)],
    ) -> Result<Vec<(Button, Shortcut)>, MouseError> {
        let mut shortcuts = Vec::new();

        for &(button, _) in buttons
            .iter()
            .filter(|(_, action)| matches!(action, ButtonAction::Shortcut))
        {
            let mut bytes = self.read_eeprom(button.shortcut_address(), MAX_READ)?;
            bytes.extend(self.read_eeprom(button.shortcut_address() + 10, MAX_READ)?);
            let bytes: [u8; 20] = bytes.try_into().expect("two reads of ten bytes");

            // A bad check is an interrupted two-frame write: leave the entry out, so
            // that set_shortcut can still repair it.
            if let Ok(Some(shortcut)) = Shortcut::from_bytes(&bytes) {
                shortcuts.push((button, shortcut));
            }
        }

        Ok(shortcuts)
    }

    /// Reads the button's current action fresh and refuses when it is unreadable or
    /// not editable.
    fn require_editable(&mut self, button: Button) -> Result<(), MouseError> {
        let current = self.read_eeprom(button.address(), 4)?;
        let action = ButtonAction::decode(current.try_into().expect("four bytes"))?;

        if !action.is_editable() {
            return Err(ProtocolError::InvalidValue("button action is not editable").into());
        }

        Ok(())
    }
}

/// Turns a rejected read into `None`: older firmware may refuse the sensor reads.
fn rejected_as_none<T>(result: Result<T, MouseError>) -> Result<Option<T>, MouseError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(MouseError::Rejected(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

fn encode_editable(action: ButtonAction) -> Result<[u8; 4], MouseError> {
    action
        .encode()
        .ok_or(ProtocolError::InvalidValue("button action is read-only").into())
}

fn pair_address(pair: usize) -> u16 {
    address::DPI_STAGES + (PAIR_LEN * pair) as u16
}

fn stage_bytes(raw: &[u8], index: usize) -> [u8; STAGE_LEN] {
    raw[index * STAGE_LEN..(index + 1) * STAGE_LEN]
        .try_into()
        .expect("four bytes")
}

#[cfg(test)]
mod tests;
