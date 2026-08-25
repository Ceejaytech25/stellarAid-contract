// contracts/escrow_tip_payment.rs
// Implements tip/bonus payment functionality for EscrowContract (closes #573)

pub struct TipPayment {
    pub escrow_id: u64,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub paid: bool,
}

pub struct EscrowTipManager {
    pub tips: Vec<TipPayment>,
    pub next_tip_id: u64,
}

impl EscrowTipManager {
    pub fn new() -> Self {
        EscrowTipManager {
            tips: Vec::new(),
            next_tip_id: 1,
        }
    }

    pub fn add_tip(&mut self, escrow_id: u64, sender: &str, recipient: &str, amount: u64) -> u64 {
        let id = self.next_tip_id;
        self.tips.push(TipPayment {
            escrow_id,
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            amount,
            paid: false,
        });
        self.next_tip_id += 1;
        id
    }

    pub fn pay_tip(&mut self, escrow_id: u64) -> Result<u64, &'static str> {
        for tip in &mut self.tips {
            if tip.escrow_id == escrow_id && !tip.paid {
                tip.paid = true;
                return Ok(tip.amount);
            }
        }
        Err("No pending tip found for this escrow")
    }

    pub fn get_pending_tips(&self) -> Vec<&TipPayment> {
        self.tips.iter().filter(|t| !t.paid).collect()
    }
}

impl Default for EscrowTipManager {
    fn default() -> Self {
        Self::new()
    }
}