// contracts/escrow_emergency_pause.rs
// Implements emergency pause functionality for EscrowContract (closes #575)

pub struct EscrowEmergencyPause {
    pub is_paused: bool,
    pub paused_by: Option<String>,
    pub pause_reason: Option<String>,
}

impl EscrowEmergencyPause {
    pub fn new() -> Self {
        EscrowEmergencyPause {
            is_paused: false,
            paused_by: None,
            pause_reason: None,
        }
    }

    pub fn pause(&mut self, admin: &str, reason: &str) -> Result<(), &'static str> {
        if self.is_paused {
            return Err("Contract is already paused");
        }
        self.is_paused = true;
        self.paused_by = Some(admin.to_string());
        self.pause_reason = Some(reason.to_string());
        Ok(())
    }

    pub fn unpause(&mut self, admin: &str) -> Result<(), &'static str> {
        if !self.is_paused {
            return Err("Contract is not paused");
        }
        self.is_paused = false;
        self.paused_by = Some(admin.to_string());
        self.pause_reason = None;
        Ok(())
    }

    pub fn check_not_paused(&self) -> Result<(), &'static str> {
        if self.is_paused {
            return Err("Contract is paused");
        }
        Ok(())
    }
}

impl Default for EscrowEmergencyPause {
    fn default() -> Self {
        Self::new()
    }
}