//! Reads and verified writes of the mouse settings.

use crate::device::{Battery, DevicePath, Inbox, Link, Model, MouseError, MouseSnapshot, Session};
use crate::protocol::{Command, Frame, ProtocolError, address};
use crate::settings::{Button, ButtonAction, DpiStage, GeneralBlock, PollingRate, Shortcut};

const STAGE_LEN: usize = 4;
const PAIR_LEN: usize = 8;
const PAIR_COUNT: usize = 4;
const MAX_READ: u8 = 10;

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

    fn read_pair(&mut self, address: u16) -> Result<[u8; PAIR_LEN], MouseError> {
        let bytes = self.read_eeprom(address, PAIR_LEN as u8)?;

        Ok(bytes.try_into().map_err(|_| ProtocolError::BadLength)?)
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
