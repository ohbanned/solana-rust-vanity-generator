use solana_sdk::signature::{Keypair, Signer};
use serde::{Serialize, Deserialize};
use log::{info, debug, error};
use tokio::sync::watch;
use tokio::time::timeout;
use std::time::Duration;
use bs58;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VanityRequest {
    pub pattern: String,
    pub position: VanityPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VanityPosition {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VanityResult {
    pub public_key: String,
    pub private_key: String,
}

pub async fn generate_with_timeout(
    req: VanityRequest,
    timeout_secs: u64,
    mut cancel_rx: watch::Receiver<bool>,
) -> Result<VanityResult, String> {
    if req.pattern.len() < 1 {
        return Err("Pattern must be at least 1 character long".to_string());
    }
    
    debug!("Starting vanity address generation for pattern: {}", req.pattern);
    
    let timeout_fut = timeout(Duration::from_secs(timeout_secs), async {
        let mut attempts = 0;
        loop {
            if *cancel_rx.borrow() {
                debug!("Generation cancelled after {} attempts", attempts);
                return Err("Generation cancelled".to_string());
            }

            attempts += 1;
            if attempts % 1000 == 0 {
                debug!("Made {} attempts", attempts);
            }

            let keypair = Keypair::new();
            let address = keypair.pubkey().to_string();
            
            let matches = match req.position {
                VanityPosition::Prefix => {
                    let prefix = &address[0..req.pattern.len()];
                    debug!("Checking prefix: {} against pattern: {}", prefix, req.pattern);
                    prefix.eq_ignore_ascii_case(&req.pattern)
                },
                VanityPosition::Suffix => {
                    let start = address.len() - req.pattern.len();
                    let suffix = &address[start..];
                    debug!("Checking suffix: {} against pattern: {}", suffix, req.pattern);
                    suffix.eq_ignore_ascii_case(&req.pattern)
                },
            };

            if matches {
                info!("Found matching address after {} attempts: {}", attempts, address);
                let result = VanityResult {
                    public_key: address,
                    private_key: bs58::encode(keypair.to_bytes()).into_string(),
                };
                debug!("Generated result: {:?}", result);
                return Ok(result);
            }
        }
    });

    match timeout_fut.await {
        Ok(result) => result,
        Err(_) => {
            debug!("Generation timed out");
            Err("Generation timed out".to_string())
        },
    }
}
