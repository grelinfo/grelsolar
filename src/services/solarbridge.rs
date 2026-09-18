//! Solar Bridge Background Service.
//! This service bridges SolarLog and Home Assistant, enabling automatic synchronization of solar production data between the two systems.

use chrono::NaiveDate;
use std::sync::Arc;
use tokio::time::{Duration, Instant, Interval, MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

use crate::integration::{homeassistant, solarlog};

/// Maximum time before an unchanged value is sent again to Home Assistant.
/// Home Assistant does not keep states set through its API across restarts.
const STATE_REFRESH_INTERVAL: Duration = Duration::from_secs(300);

/// Time a sync must keep failing before its sensor is marked unavailable,
/// so that Home Assistant does not show a stale value as if it were live.
const STALE_AFTER: Duration = Duration::from_secs(60);

/// Time between two error logs while a sync keeps failing.
const FAILURE_LOG_INTERVAL: Duration = Duration::from_secs(3600);

pub struct SolarBridgeBackgroundService {
    solarlog: Arc<solarlog::Client>,
    homeassistant: Arc<homeassistant::Client>,
    sync_power_interval: Duration,
    sync_energy_interval: Duration,
    sync_status_interval: Duration,
}

impl SolarBridgeBackgroundService {
    /// Creates a new instance of `SolarService`.
    pub fn new(
        solarlog: Arc<solarlog::Client>,
        homeassistant: Arc<homeassistant::Client>,
        sync_power_interval: Duration,
        sync_energy_interval: Duration,
        sync_status_interval: Duration,
    ) -> Self {
        SolarBridgeBackgroundService {
            solarlog,
            homeassistant,
            sync_power_interval,
            sync_energy_interval,
            sync_status_interval,
        }
    }

    /// Run the background service to synchronize data between SolarLog and Home Assistant.
    pub async fn run(&self, token: CancellationToken) {
        tokio::join!(
            self.sync_solar_power_task(self.sync_power_interval, token.clone()),
            self.sync_solar_energy_task(self.sync_energy_interval, token.clone()),
            self.sync_solar_status_task(self.sync_status_interval, token.clone())
        );
    }

    /// Periodically retrieves the current power from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for current power data.
    async fn sync_solar_power_task(&self, period: Duration, token: CancellationToken) {
        let mut state = SyncState::new("power");
        let mut interval = ticker(period);

        loop {
            tokio::select! {
                _ = interval.tick() => {},
                _ = token.cancelled() => {
                    log::debug!("sync_solar_power_task: shutting down");
                    return;
                }
            }
            match self.sync_solar_power(state.last().copied()).await {
                Ok(value) => state.succeeded(value),
                Err(e) => {
                    if state.failed(&e) {
                        state.marked_unavailable(
                            self.homeassistant
                                .set_solar_current_power_unavailable()
                                .await,
                        );
                    }
                }
            }
        }
    }

    /// Periodically retrieves the inverter status from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for inverter status data.
    async fn sync_solar_energy_task(&self, period: Duration, token: CancellationToken) {
        let mut state = SyncState::new("energy");
        let mut interval = ticker(period);

        loop {
            tokio::select! {
                _ = interval.tick() => {},
                _ = token.cancelled() => {
                    log::debug!("sync_solar_energy_task: shutting down");
                    return;
                }
            }
            match self.sync_solar_energy(state.last().copied()).await {
                Ok(value) => state.succeeded(value),
                Err(e) => {
                    if state.failed(&e) {
                        state.marked_unavailable(
                            self.homeassistant.set_solar_energy_unavailable().await,
                        );
                    }
                }
            }
        }
    }

    /// Periodically retrieves the inverter status from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for inverter status data.
    async fn sync_solar_status_task(&self, period: Duration, token: CancellationToken) {
        let mut state = SyncState::new("status");
        let mut interval = ticker(period);

        loop {
            tokio::select! {
                _ = interval.tick() => {},
                _ = token.cancelled() => {
                    log::debug!("sync_solar_status_task: shutting down");
                    return;
                }
            }
            match self.sync_solar_status(state.last()).await {
                Ok(value) => state.succeeded(value),
                Err(e) => {
                    if state.failed(&e) {
                        state.marked_unavailable(
                            self.homeassistant.set_solar_status_unavailable().await,
                        );
                    }
                }
            }
        }
    }

    /// Synchronizes the current solar power with Home Assistant.
    pub async fn sync_solar_power(
        &self,
        last_power: Option<i64>,
    ) -> Result<Option<i64>, anyhow::Error> {
        let power = self.solarlog.get_current_power().await?;
        if last_power == Some(power) {
            return Ok(Some(power));
        }
        self.homeassistant.set_solar_current_power(power).await?;
        Ok(Some(power))
    }

    /// Synchronizes the solar energy produced today with Home Assistant.
    pub async fn sync_solar_energy(
        &self,
        last_value: Option<(NaiveDate, i64)>,
    ) -> Result<Option<(NaiveDate, i64)>, anyhow::Error> {
        let value = self.solarlog.get_energy_of_last_day().await?;
        if last_value == Some(value) {
            return Ok(Some(value));
        }
        self.homeassistant.set_solar_energy(value.1).await?;
        Ok(Some(value))
    }

    /// Synchronizes the SolarLog device status with Home Assistant.
    pub async fn sync_solar_status(
        &self,
        last_status: Option<&solarlog::InverterStatus>,
    ) -> Result<Option<solarlog::InverterStatus>, anyhow::Error> {
        let status = self.solarlog.get_status().await?;
        if last_status == Some(&status) {
            return Ok(Some(status));
        }
        let status_str = status.to_string();
        self.homeassistant.set_solar_status(&status_str).await?;
        Ok(Some(status))
    }
}

/// Create an interval that skips missed ticks instead of bursting to catch up.
fn ticker(period: Duration) -> Interval {
    let mut interval = interval(period);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    interval
}

/// Last synced value and failure tracking of one sync task.
struct SyncState<T> {
    name: &'static str,
    last: Option<T>,
    refreshed_at: Instant,
    failing_since: Option<Instant>,
    logged_at: Instant,
    unavailable: bool,
}

impl<T> SyncState<T> {
    fn new(name: &'static str) -> Self {
        let now = Instant::now();
        Self {
            name,
            last: None,
            refreshed_at: now,
            failing_since: None,
            logged_at: now,
            unavailable: false,
        }
    }

    /// Last synced value, forgotten every `STATE_REFRESH_INTERVAL` so that
    /// the next sync sends it again even if it did not change.
    fn last(&mut self) -> Option<&T> {
        if self.refreshed_at.elapsed() >= STATE_REFRESH_INTERVAL {
            self.last = None;
            self.refreshed_at = Instant::now();
        }
        self.last.as_ref()
    }

    /// Record a successful sync.
    fn succeeded(&mut self, value: Option<T>) {
        if let Some(since) = self.failing_since.take() {
            log::info!(
                "Sync {} recovered after {}",
                self.name,
                rounded(since.elapsed())
            );
        }
        self.unavailable = false;
        self.last = value;
    }

    /// Record a failed sync, logging it once when it starts and then once
    /// every `FAILURE_LOG_INTERVAL`.
    /// Returns `true` if the sensor should now be marked unavailable.
    fn failed(&mut self, error: &anyhow::Error) -> bool {
        let now = Instant::now();
        let since = match self.failing_since {
            Some(since) => {
                if now - self.logged_at >= FAILURE_LOG_INTERVAL {
                    log::error!(
                        "Sync {} still failing for {}: {error}",
                        self.name,
                        rounded(now - since)
                    );
                    self.logged_at = now;
                } else {
                    log::debug!("Sync {} failed: {error}", self.name);
                }
                since
            }
            None => {
                log::error!("Sync {} failed: {error}", self.name);
                self.failing_since = Some(now);
                self.logged_at = now;
                now
            }
        };
        !self.unavailable && now - since >= STALE_AFTER
    }

    /// Record the result of marking the sensor unavailable in Home Assistant.
    /// On failure it is tried again after the next failed sync.
    fn marked_unavailable(&mut self, result: homeassistant::Result<()>) {
        match result {
            Ok(()) => {
                log::warn!("Sensor {} marked unavailable in Home Assistant", self.name);
                self.unavailable = true;
                // Send the value again as soon as the sync recovers
                self.last = None;
            }
            Err(e) => log::debug!("Cannot mark sensor {} unavailable: {e}", self.name),
        }
    }
}

/// Round a duration to whole seconds for logging.
fn rounded(duration: Duration) -> humantime::FormattedDuration {
    humantime::format_duration(Duration::from_secs(duration.as_secs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error() -> anyhow::Error {
        anyhow::anyhow!("boom")
    }

    #[tokio::test(start_paused = true)]
    async fn test_last_is_kept_before_refresh_interval() {
        let mut state = SyncState::new("test");
        state.succeeded(Some(42));

        tokio::time::advance(STATE_REFRESH_INTERVAL / 2).await;

        assert_eq!(state.last(), Some(&42));
    }

    #[tokio::test(start_paused = true)]
    async fn test_last_is_forgotten_after_refresh_interval() {
        let mut state = SyncState::new("test");
        state.succeeded(Some(42));

        tokio::time::advance(STATE_REFRESH_INTERVAL).await;

        assert_eq!(state.last(), None);
    }

    #[tokio::test(start_paused = true)]
    async fn test_failed_marks_unavailable_only_after_stale_delay() {
        let mut state: SyncState<i64> = SyncState::new("test");

        assert!(!state.failed(&error()));
        tokio::time::advance(STALE_AFTER / 2).await;
        assert!(!state.failed(&error()));
        tokio::time::advance(STALE_AFTER / 2).await;
        assert!(state.failed(&error()));
    }

    #[tokio::test(start_paused = true)]
    async fn test_marked_unavailable_is_done_once_and_forgets_last() {
        let mut state = SyncState::new("test");
        state.succeeded(Some(42));
        state.failed(&error());
        tokio::time::advance(STALE_AFTER).await;

        state.marked_unavailable(Ok(()));

        assert!(!state.failed(&error()));
        assert_eq!(state.last(), None);
    }

    #[tokio::test(start_paused = true)]
    async fn test_marked_unavailable_is_retried_after_error() {
        let mut state: SyncState<i64> = SyncState::new("test");
        state.failed(&error());
        tokio::time::advance(STALE_AFTER).await;

        state.marked_unavailable(Err(homeassistant::Error::RequestRejected));

        assert!(state.failed(&error()));
    }

    #[tokio::test(start_paused = true)]
    async fn test_succeeded_resets_failure_tracking() {
        let mut state = SyncState::new("test");
        state.failed(&error());
        tokio::time::advance(STALE_AFTER).await;
        state.marked_unavailable(Ok(()));

        state.succeeded(Some(42));

        assert_eq!(state.last(), Some(&42));
        assert!(!state.failed(&error()));
    }

    #[tokio::test(start_paused = true)]
    async fn test_ticker_skips_missed_ticks() {
        let mut interval = ticker(Duration::from_secs(1));
        interval.tick().await;

        tokio::time::advance(Duration::from_millis(3500)).await;
        interval.tick().await;
        let before = Instant::now();
        interval.tick().await;

        assert_eq!(Instant::now() - before, Duration::from_millis(500));
    }
}
