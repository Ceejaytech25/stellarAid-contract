// contracts/platform_config_upgrade.rs
// Implements contract upgrade mechanism for PlatformConfig (closes #574)

pub struct UpgradeProposal {
    pub proposal_id: u64,
    pub new_implementation: String,
    pub proposed_by: String,
    pub approved: bool,
}

pub struct PlatformConfigUpgrade {
    pub current_implementation: String,
    pub upgrade_admin: String,
    pub pending_proposal: Option<UpgradeProposal>,
    pub next_proposal_id: u64,
}

impl PlatformConfigUpgrade {
    pub fn new(implementation: &str, admin: &str) -> Self {
        PlatformConfigUpgrade {
            current_implementation: implementation.to_string(),
            upgrade_admin: admin.to_string(),
            pending_proposal: None,
            next_proposal_id: 1,
        }
    }

    pub fn propose_upgrade(&mut self, proposer: &str, new_impl: &str) -> u64 {
        let id = self.next_proposal_id;
        self.pending_proposal = Some(UpgradeProposal {
            proposal_id: id,
            new_implementation: new_impl.to_string(),
            proposed_by: proposer.to_string(),
            approved: false,
        });
        self.next_proposal_id += 1;
        id
    }

    pub fn approve_upgrade(&mut self, admin: &str) -> Result<(), &'static str> {
        if admin != self.upgrade_admin {
            return Err("Only admin can approve upgrades");
        }
        match &mut self.pending_proposal {
            Some(p) => { p.approved = true; Ok(()) }
            None => Err("No pending upgrade proposal"),
        }
    }

    pub fn execute_upgrade(&mut self) -> Result<String, &'static str> {
        match &self.pending_proposal {
            Some(p) if p.approved => {
                self.current_implementation = p.new_implementation.clone();
                self.pending_proposal = None;
                Ok(self.current_implementation.clone())
            }
            Some(_) => Err("Upgrade not approved"),
            None => Err("No pending upgrade proposal"),
        }
    }
}