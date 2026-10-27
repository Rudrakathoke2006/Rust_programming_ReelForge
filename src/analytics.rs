use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReelRecord {
    pub id: String,
    pub user_id: String,
    pub timestamp: u64,
    pub date_str: String,
    pub event: String,
    pub motion: String,
    pub photos_count: usize,
    pub duration_secs: f32,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserProfile {
    pub user_id: String,
    pub total_reels: usize,
    pub first_seen: u64,
    pub last_seen: u64,
    pub first_seen_str: String,
    pub last_seen_str: String,
    pub favorite_event: String,
    pub event_counts: HashMap<String, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AnalyticsStore {
    pub total_reels_generated: usize,
    pub users: HashMap<String, UserProfile>,
    pub recent_reels: Vec<ReelRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_reels: usize,
    pub total_users: usize,
    pub event_breakdown: HashMap<String, usize>,
    pub motion_breakdown: HashMap<String, usize>,
    pub users: Vec<UserProfile>,
    pub recent_reels: Vec<ReelRecord>,
}

pub struct AnalyticsTracker {
    path: PathBuf,
    store: RwLock<AnalyticsStore>,
}

impl AnalyticsTracker {
    pub fn new(path: impl Into<PathBuf>) -> Arc<Self> {
        let p = path.into();
        let store = Self::load(&p).unwrap_or_default();
        Arc::new(Self {
            path: p,
            store: RwLock::new(store),
        })
    }

    fn load(path: &Path) -> Option<AnalyticsStore> {
        if path.exists() {
            let data = fs::read_to_string(path).ok()?;
            serde_json::from_str(&data).ok()
        } else {
            None
        }
    }

    fn save_internal(path: &Path, store: &AnalyticsStore) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(store) {
            let _ = fs::write(path, json);
        }
    }

    pub fn record_reel(
        &self,
        raw_user: &str,
        event: &str,
        motion: &str,
        photos_count: usize,
        duration: f32,
        title: Option<&str>,
    ) {
        let user_id = if raw_user.trim().is_empty() {
            "Guest Creator".to_string()
        } else {
            raw_user.trim().to_string()
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let date_str = format_epoch(now);

        let record = ReelRecord {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.clone(),
            timestamp: now,
            date_str: date_str.clone(),
            event: event.to_string(),
            motion: motion.to_string(),
            photos_count,
            duration_secs: duration,
            title: title.map(|s| s.to_string()),
        };

        if let Ok(mut store) = self.store.write() {
            store.total_reels_generated += 1;

            let u = store.users.entry(user_id.clone()).or_insert_with(|| UserProfile {
                user_id: user_id.clone(),
                total_reels: 0,
                first_seen: now,
                last_seen: now,
                first_seen_str: date_str.clone(),
                last_seen_str: date_str.clone(),
                favorite_event: event.to_string(),
                event_counts: HashMap::new(),
            });

            u.total_reels += 1;
            u.last_seen = now;
            u.last_seen_str = date_str;
            *u.event_counts.entry(event.to_string()).or_insert(0) += 1;

            // Update favorite event
            if let Some((best_event, _)) = u.event_counts.iter().max_by_key(|e| e.1) {
                u.favorite_event = best_event.clone();
            }

            // Keep recent 100 reels in history
            store.recent_reels.insert(0, record);
            if store.recent_reels.len() > 100 {
                store.recent_reels.truncate(100);
            }

            Self::save_internal(&self.path, &store);
        }
    }

    pub fn get_summary(&self) -> AnalyticsSummary {
        let store = self.store.read().unwrap();
        let mut event_breakdown: HashMap<String, usize> = HashMap::new();
        let mut motion_breakdown: HashMap<String, usize> = HashMap::new();

        for r in &store.recent_reels {
            *event_breakdown.entry(r.event.clone()).or_insert(0) += 1;
            *motion_breakdown.entry(r.motion.clone()).or_insert(0) += 1;
        }

        let mut users_list: Vec<UserProfile> = store.users.values().cloned().collect();
        // Sort users by most reels generated descending
        users_list.sort_by(|a, b| b.total_reels.cmp(&a.total_reels));

        AnalyticsSummary {
            total_reels: store.total_reels_generated,
            total_users: store.users.len(),
            event_breakdown,
            motion_breakdown,
            users: users_list,
            recent_reels: store.recent_reels.clone(),
        }
    }
}

pub fn format_epoch(epoch: u64) -> String {
    let secs_per_day = 86400u64;
    let days_since_epoch = epoch / secs_per_day;
    let day_secs = epoch % secs_per_day;
    let hours = day_secs / 3600;
    let mins = (day_secs % 3600) / 60;
    let secs = day_secs % 60;

    let z = days_since_epoch as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", y, m, d, hours, mins, secs)
}
