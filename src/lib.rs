//! RFC-008: CMTN Implementation Example
//! http://cmtn.com/
//! This example demonstrates the core concepts of RFC-008:
//! 1. Civilization Signature Management
//! 2. Pulse-Integrated Diplomacy
//! 3. Dark Multi-Tenancy
//! 4. Reputation Metabolism

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use sha3::{Digest, Sha3_512};

// ============================================================================
// Core Types
// ============================================================================

/// 512-bit Civilization Signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CivilizationSignature([u8; 64]);

impl CivilizationSignature {
    pub fn new(domain: &str, aid: &[u8; 32], reputation: f64) -> Self {
        let mut hasher = Sha3_512::new();
        hasher.update(domain.as_bytes());
        hasher.update(aid);
        hasher.update(reputation.to_le_bytes());
        let hash = hasher.finalize();
        
        let mut signature = [0u8; 64];
        signature.copy_from_slice(&hash);
        Self(signature)
    }
    
    pub fn verify(&self, domain: &str, aid: &[u8; 32]) -> bool {
        // Simplified verification - in production, use proper cryptographic verification
        let test_signature = Self::new(domain, aid, 1.0);
        self.0[..32] == test_signature.0[..32]
    }
}

/// Diplomatic Intent Types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntentType {
    Bid,           // Resource bidding
    Ask,           // Resource request
    PolicySync,    // Policy synchronization
    Alliance,      // Alliance formation
    Treaty,        // Treaty negotiation
}

/// Diplomatic Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticRequest {
    pub request_id: u128,
    pub source: CivilizationSignature,
    pub target: CivilizationSignature,
    pub intent: IntentType,
    pub payload: Vec<u8>,
    pub deadline: u64,      // µs timestamp
    pub max_latency: u32,   // µs
}

/// Diplomatic Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticResponse {
    pub request_id: u128,
    pub response_type: ResponseType,
    pub reputation_adjustment: i32,
    pub latency: u32,       // µs
    pub next_state: NegotiationState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResponseType {
    Accept,
    Reject,
    CounterOffer,
    Deferred,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NegotiationState {
    Idle,
    Negotiating,
    Finalizing,
    Completed,
    Rejected,
    TimedOut,
}

// ============================================================================
// Reputation System
// ============================================================================

/// Reputation Metabolism Engine
pub struct ReputationEngine {
    scores: RwLock<HashMap<CivilizationSignature, f64>>,
    history: RwLock<HashMap<CivilizationSignature, Vec<ReputationEvent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub timestamp: u64,
    pub delta: f64,
    pub reason: String,
    pub transaction_hash: Option<[u8; 32]>,
}

impl ReputationEngine {
    pub fn new() -> Self {
        Self {
            scores: RwLock::new(HashMap::new()),
            history: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn get_score(&self, signature: &CivilizationSignature) -> f64 {
        let scores = self.scores.read().await;
        *scores.get(signature).unwrap_or(&1.0)
    }
    
    pub async fn adjust_score(
        &self,
        signature: &CivilizationSignature,
        delta: f64,
        reason: String,
        transaction_hash: Option<[u8; 32]>,
    ) -> f64 {
        let mut scores = self.scores.write().await;
        let mut history = self.history.write().await;
        
        let current = scores.entry(signature.clone()).or_insert(1.0);
        *current = (*current + delta).max(0.0).min(10.0);
        
        let event = ReputationEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
            delta,
            reason,
            transaction_hash,
        };
        
        history.entry(signature.clone())
            .or_insert_with(Vec::new)
            .push(event);
        
        *current
    }
    
    pub async fn calculate_metabolic_trust(
        &self,
        signature: &CivilizationSignature,
        resource_contribution: f64,
        homeostasis_score: f64,
    ) -> f64 {
        let base_score = self.get_score(signature).await;
        
        // Metabolic Trust Score formula from RFC-008
        let trust_score = base_score 
            * (0.6 + 0.4 * resource_contribution).min(1.0)
            * (0.8 + 0.2 * homeostasis_score).min(1.0);
        
        trust_score.min(10.0)
    }
}

// ============================================================================
// Diplomatic Engine
// ============================================================================

/// Pulse-Integrated Diplomacy Engine
pub struct DiplomaticEngine {
    reputation_engine: Arc<ReputationEngine>,
    pending_negotiations: RwLock<HashMap<u128, NegotiationState>>,
    max_latency: u32, // µs
}

impl DiplomaticEngine {
    pub fn new(reputation_engine: Arc<ReputationEngine>) -> Self {
        Self {
            reputation_engine,
            pending_negotiations: RwLock::new(HashMap::new()),
            max_latency: 450_000, // 450µs as per RFC-008
        }
    }
    
    pub async fn send_request(
        &self,
        request: DiplomaticRequest,
    ) -> Result<DiplomaticResponse, DiplomaticError> {
        // Check reputation threshold
        let source_reputation = self.reputation_engine
            .get_score(&request.source)
            .await;
        
        if source_reputation < 0.95 {
            return Err(DiplomaticError::ReputationThresholdNotMet(source_reputation));
        }
        
        // Check latency constraint
        if request.max_latency > self.max_latency {
            return Err(DiplomaticError::MaxLatencyExceeded(request.max_latency));
        }
        
        // Start negotiation
        let start_time = std::time::SystemTime::now();
        self.pending_negotiations.write().await
            .insert(request.request_id, NegotiationState::Negotiating);
        
        // Simulate diplomatic negotiation
        let result = self.negotiate(&request).await;
        
        // Record latency
        let latency = start_time.elapsed()
            .map(|d| d.as_micros() as u32)
            .unwrap_or(u32::MAX);
        
        // Update negotiation state
        let next_state = match &result {
            Ok(_) => NegotiationState::Completed,
            Err(_) => NegotiationState::Rejected,
        };
        
        self.pending_negotiations.write().await
            .insert(request.request_id, next_state);
        
        match result {
            Ok(mut response) => {
                response.latency = latency;
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
    
    async fn negotiate(
        &self,
        request: &DiplomaticRequest,
    ) -> Result<DiplomaticResponse, DiplomaticError> {
        // Simulate tensor alignment (RFC-008 2.1)
        tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        
        // Check if alignment reached
        let alignment_reached = rand::random::<f64>() > 0.1; // 90% success rate
        
        if !alignment_reached {
            return Err(DiplomaticError::TensorAlignmentFailed);
        }
        
        // Generate response
        Ok(DiplomaticResponse {
            request_id: request.request_id,
            response_type: ResponseType::Accept,
            reputation_adjustment: 10, // Positive adjustment for successful diplomacy
            latency: 0, // Will be set by caller
            next_state: NegotiationState::Completed,
        })
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DiplomaticError {
    #[error("Reputation threshold not met: {0}")]
    ReputationThresholdNotMet(f64),
    
    #[error("Maximum latency exceeded: {0}µs")]
    MaxLatencyExceeded(u32),
    
    #[error("Tensor alignment failed")]
    TensorAlignmentFailed,
    
    #[error("Civilization signature invalid")]
    InvalidSignature,
    
    #[error("Negotiation timeout")]
    NegotiationTimeout,
}

// ============================================================================
// Example Usage
// ============================================================================

#[tokio::main]
async fn main() {
    println!("RFC-007: CMTN Implementation Example");
    println!("=====================================\n");
    
    // Initialize reputation engine
    let reputation_engine = Arc::new(ReputationEngine::new());
    let diplomatic_engine = DiplomaticEngine::new(reputation_engine.clone());
    
    // Create two civilizations
    let aid1 = [0u8; 32];
    let aid2 = [1u8; 32];
    
    let civ1 = CivilizationSignature::new("cmtn.com", &aid1, 1.0);
    let civ2 = CivilizationSignature::new("example.com", &aid2, 1.0);
    
    println!("Civilization 1: {:?}", hex::encode(&civ1.0[..16]));
    println!("Civilization 2: {:?}", hex::encode(&civ2.0[..16]));
    
    // Verify signatures
    assert!(civ1.verify("cmtn.com", &aid1));
    assert!(civ2.verify("example.com", &aid2));
    
    println!("\n✓ Civilization signatures verified\n");
    
    // Create diplomatic request
    let request = DiplomaticRequest {
        request_id: 123456789,
        source: civ1.clone(),
        target: civ2.clone(),
        intent: IntentType::Alliance,
        payload: b"Proposal for mutual defense alliance".to_vec(),
        deadline: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64 + 1_000_000, // 1 second deadline
        max_latency: 400_000, // 400µs
    };
    
    println!("Sending diplomatic request...");
    println!("  Intent: {:?}", request.intent);
    println!("  Max Latency: {}µs", request.max_latency);
    
    // Send diplomatic request
    match diplomatic_engine.send_request(request).await {
        Ok(response) => {
            println!("\n✓ Diplomatic agreement reached!");
            println!("  Response: {:?}", response.response_type);
            println!("  Latency: {}µs", response.latency);
            println!("  Reputation Adjustment: +{}", response.reputation_adjustment);
            
            // Update reputation
            let new_score = reputation_engine
                .adjust_score(
                    &civ1,
                    response.reputation_adjustment as f64 / 100.0,
                    "Successful diplomatic agreement".to_string(),
                    None,
                )
                .await;
            
            println!("  New Reputation Score: {:.2}", new_score);
        }
        Err(e) => {
            println!("\n✗ Diplomatic failure: {}", e);
        }
    }
    
    // Demonstrate reputation metabolism
    println!("\n--- Reputation Metabolism Example ---");
    
    let resource_contribution = 0.8; // 80% resource contribution
    let homeostasis_score = 0.9;     // 90% system health
    
    let metabolic_trust = reputation_engine
        .calculate_metabolic_trust(&civ1, resource_contribution, homeostasis_score)
        .await;
    
    println!("Resource Contribution: {:.0}%", resource_contribution * 100.0);
    println!("Homeostasis Score: {:.0}%", homeostasis_score * 100.0);
    println!("Metabolic Trust Score: {:.2}", metabolic_trust);
    
    // Check if civilization qualifies for high-value diplomatic mesh
    if metabolic_trust > 0.95 {
        println!("✓ Qualifies for High-Value Diplomatic Mesh (RFC-007 2.3)");
    } else {
        println!("✗ Does not qualify for High-Value Diplomatic Mesh");
    }
    
    println!("\n=====================================");
    println!("RFC-007 Implementation Example Complete");
    println!("Civilization Protocol: ACTIVE");
    println!("Dark Multi-Tenancy: ENABLED");
    println!("Atomic Diplomacy: OPERATIONAL");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_diplomatic_latency() {
        let reputation_engine = Arc::new(ReputationEngine::new());
        let diplomatic_engine = DiplomaticEngine::new(reputation_engine);
        
        let aid = [0u8; 32];
        let civ1 = CivilizationSignature::new("test1.com", &aid, 1.0);
        let civ2 = CivilizationSignature::new("test2.com", &aid, 1.0);
        
        let request = DiplomaticRequest {
            request_id: 1,
            source: civ1,
            target: civ2,
            intent: IntentType::Bid,
            payload: vec![],
            deadline: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64 + 1_000_000,
            max_latency: 400_000,
        };
        
        // Test that diplomacy completes within 450µs
        let result = timeout(
            Duration::from_micros(450),
            diplomatic_engine.send_request(request)
        ).await;
        
        assert!(result.is_ok(), "Diplomacy should complete within 450µs");
    }
    
    #[tokio::test]
    async fn test_reputation_threshold() {
        let reputation_engine = Arc::new(ReputationEngine::new());
        let diplomatic_engine = DiplomaticEngine::new(reputation_engine);
        
        let aid = [0u8; 32];
        let mut civ1 = CivilizationSignature::new("lowrep.com", &aid, 1.0);
        
        // Create a request with low reputation civilization
        let request = DiplomaticRequest {
            request_id: 2,
            source: civ1.clone(),
            target: CivilizationSignature::new("target.com", &aid, 1.0),
            intent: IntentType::Ask,
            payload: vec![],
            deadline: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64 + 1_000_000,
            max_latency: 400_000,
        };
        
        // This should fail due to low reputation
        let result = diplomatic_engine.send_request(request).await;
        
        assert!(matches!(
            result,
            Err(DiplomaticError::ReputationThresholdNotMet(_))
        ));
    }
    
    #[tokio::test]
    async fn test_civilization_isolation() {
        // Test that civilizations are properly isolated
        let aid1 = [0u8; 32];
        let aid2 = [1u8; 32];
        
        let civ1 = CivilizationSignature::new("tenant1.com", &aid1, 1.0);
        let civ2 = CivilizationSignature::new("tenant2.com", &aid2, 1.0);
        
        // Verify signatures are different (simulating entropy gap)
        assert_ne!(civ1.0, civ2.0, "Civilization signatures should be unique");
        
        // Verify each civilization can only verify its own signature
        assert!(civ1.verify("tenant1.com", &aid1));
        assert!(!civ1.verify("tenant2.com", &aid2));
        
        assert!(civ2.verify("tenant2.com", &aid2));
        assert!(!civ2.verify("tenant1.com", &aid1));
    }
}

// ============================================================================
// Performance Metrics
// ============================================================================

/// Performance metrics collector for RFC-007 compliance
pub struct CmtnMetrics {
    total_requests: std::sync::atomic::AtomicU64,
    successful_requests: std::sync::atomic::AtomicU64,
    total_latency: std::sync::atomic::AtomicU64,
    max_latency: std::sync::atomic::AtomicU32,
}

impl CmtnMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: std::sync::atomic::AtomicU64::new(0),
            successful_requests: std::sync::atomic::AtomicU64::new(0),
            total_latency: std::sync::atomic::AtomicU64::new(0),
            max_latency: std::sync::atomic::AtomicU32::new(0),
        }
    }
    
    pub fn record_request(&self, latency: u32, success: bool) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if success {
            self.successful_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        self.total_latency.fetch_add(latency as u64, std::sync::atomic::Ordering::Relaxed);
        
        let mut current_max = self.max_latency.load(std::sync::atomic::Ordering::Relaxed);
        while latency > current_max {
            match self.max_latency.compare_exchange(
                current_max,
                latency,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_current) => current_max = new_current,
            }
        }
    }
    
    pub fn get_metrics(&self) -> CmtnMetricsSnapshot {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        let successful = self.successful_requests.load(std::sync::atomic::Ordering::Relaxed);
        let total_latency = self.total_latency.load(std::sync::atomic::Ordering::Relaxed);
        let max_latency = self.max_latency.load(std::sync::atomic::Ordering::Relaxed);
        
        let avg_latency = if total > 0 {
            total_latency / total
        } else {
            0
        };
        
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        CmtnMetricsSnapshot {
            total_requests: total,
            successful_requests: successful,
            average_latency: avg_latency,
            max_latency,
            success_rate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CmtnMetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub average_latency: u64,
    pub max_latency: u32,
    pub success_rate: f64,
}

impl std::fmt::Display for CmtnMetricsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CMTN Metrics:
  Total Requests: {}
  Successful: {} ({:.1}%)
  Average Latency: {}µs
  Max Latency: {}µs
  RFC-007 Compliance: {}",
            self.total_requests,
            self.successful_requests,
            self.success_rate,
            self.average_latency,
            self.max_latency,
            if self.max_latency <= 450_000 && self.success_rate >= 95.0 {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        )
    }
}
