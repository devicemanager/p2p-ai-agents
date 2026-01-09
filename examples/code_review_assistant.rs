//! Code Review Assistant Example
//!
//! Demonstrates a distributed expert network for code review with:
//! - Multi-expert consultation
//! - Transparent reasoning
//! - Confidence scoring
//! - Detailed explanations

use p2p_ai_agents::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct CodeReviewQuery {
    language: String,
    domain: String,
    code: String,
    focus_areas: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReviewIssue {
    severity: String,
    line: usize,
    description: String,
    reasoning: String,
    suggested_fix: String,
    confidence: f32,
    rule_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeReviewResult {
    issues: Vec<ReviewIssue>,
    overall_score: u8,
    expert_consensus: HashMap<String, String>,
    reasoning_trace: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔍 Distributed Code Review Assistant");
    println!("====================================\n");

    // Example code to review
    let code_sample = r#"
use tokio::sync::Mutex;
use std::sync::Arc;

async fn process_data(data: Arc<Mutex<Vec<i32>>>) {
    let mut d = data.lock().await;
    // Simulated work
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    d.push(42);
}

#[tokio::main]
async fn main() {
    let data = Arc::new(Mutex::new(vec![]));
    
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let d = data.clone();
            tokio::spawn(async move {
                process_data(d).await;
            })
        })
        .collect();
    
    for h in handles {
        h.await.unwrap();
    }
}
"#;

    // Create query
    let query = CodeReviewQuery {
        language: "Rust".to_string(),
        domain: "async-concurrency".to_string(),
        code: code_sample.to_string(),
        focus_areas: vec![
            "deadlocks".to_string(),
            "race-conditions".to_string(),
            "performance".to_string(),
        ],
    };

    println!("📝 Submitting code for review...");
    println!("Language: {}", query.language);
    println!("Domain: {}", query.domain);
    println!("Focus: {:?}\n", query.focus_areas);

    // Simulate expert network review
    let result = simulate_expert_review(query).await?;

    // Display results
    println!("📊 Review Results");
    println!("================");
    println!("Overall Score: {}/100", result.overall_score);
    println!("Issues Found: {}\n", result.issues.len());

    for (i, issue) in result.issues.iter().enumerate() {
        println!("Issue #{}: {} ({})", i + 1, issue.severity, issue.confidence);
        println!("  Line: {}", issue.line);
        println!("  Problem: {}", issue.description);
        println!("  Why: {}", issue.reasoning);
        println!("  Fix: {}", issue.suggested_fix);
        println!("  Rule: {}\n", issue.rule_id);
    }

    println!("🔍 Expert Consensus:");
    for (expert, opinion) in &result.expert_consensus {
        println!("  {}: {}", expert, opinion);
    }

    println!("\n📜 Reasoning Trace:");
    for (i, step) in result.reasoning_trace.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }

    Ok(())
}

/// Simulate a multi-expert review process
/// In production, this would query actual expert nodes
async fn simulate_expert_review(query: CodeReviewQuery) -> Result<CodeReviewResult> {
    // Simulate expert analysis
    let issues = vec![
        ReviewIssue {
            severity: "WARNING".to_string(),
            line: 7,
            description: "Potential deadlock: Holding lock across await point".to_string(),
            reasoning: "The Mutex lock is held while awaiting, which blocks other tasks. \
                       If multiple tasks do this simultaneously, they can deadlock."
                .to_string(),
            suggested_fix: "Move the lock() call after the await, or use a smaller critical section"
                .to_string(),
            confidence: 0.92,
            rule_id: "RUST-ASYNC-001".to_string(),
        },
        ReviewIssue {
            severity: "INFO".to_string(),
            line: 17,
            description: "Consider using RwLock for read-heavy workloads".to_string(),
            reasoning: "If reads are more frequent than writes, RwLock allows multiple \
                       concurrent readers, improving performance."
                .to_string(),
            suggested_fix: "Replace Mutex with RwLock if appropriate".to_string(),
            confidence: 0.75,
            rule_id: "RUST-PERF-042".to_string(),
        },
    ];

    let expert_consensus = HashMap::from([
        (
            "rust-async-expert-1".to_string(),
            "Critical deadlock risk identified".to_string(),
        ),
        (
            "rust-async-expert-2".to_string(),
            "Confirmed - pattern matches known anti-pattern".to_string(),
        ),
        (
            "rust-perf-expert-1".to_string(),
            "Performance optimization available".to_string(),
        ),
    ]);

    let reasoning_trace = vec![
        "Query routed to rust-async domain experts".to_string(),
        "Expert-1: Scanned for async patterns with locks".to_string(),
        "Expert-1: Rule RUST-ASYNC-001 matched at line 7".to_string(),
        "Expert-2: Validated finding against known patterns database".to_string(),
        "Expert-2: Confirmed with 92% confidence".to_string(),
        "Expert-3: Analyzed lock contention patterns".to_string(),
        "Expert-3: Suggested RwLock optimization".to_string(),
        "Consensus: 3/3 experts agree on deadlock risk".to_string(),
    ];

    Ok(CodeReviewResult {
        issues,
        overall_score: 75,
        expert_consensus,
        reasoning_trace,
    })
}
