// Wayland idle detection using ext-idle-notify-v1 protocol
// This implementation uses a background thread to maintain a persistent Wayland connection
// and monitor idle state continuously.

use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use wayrs_client::{protocol::WlSeat, Connection, EventCtx, IoMode};
use wayrs_protocols::ext_idle_notify_v1::{
    ext_idle_notification_v1, ExtIdleNotificationV1, ExtIdleNotifierV1,
};
use wayrs_utils::seats::{SeatHandler, Seats};

pub struct WaylandIdleDetector {
    is_idle: Arc<Mutex<bool>>,
    last_activity: Arc<Mutex<Instant>>,
    #[allow(dead_code)]
    timeout_seconds: u64,
}

impl WaylandIdleDetector {
    pub fn new(timeout_seconds: u64) -> Self {
        let is_idle = Arc::new(Mutex::new(false));
        let last_activity = Arc::new(Mutex::new(Instant::now()));

        let detector = Self {
            is_idle: Arc::clone(&is_idle),
            last_activity: Arc::clone(&last_activity),
            timeout_seconds,
        };

        // Spawn background thread to monitor idle state
        let is_idle_clone = Arc::clone(&is_idle);
        let last_activity_clone = Arc::clone(&last_activity);
        let timeout_ms = (timeout_seconds * 1000) as u32;

        thread::spawn(move || {
            if let Err(e) = Self::monitor_idle_state(is_idle_clone, last_activity_clone, timeout_ms)
            {
                eprintln!("Wayland idle monitor thread error: {}", e);
            }
        });

        detector
    }

    /// Background thread function that monitors idle state
    fn monitor_idle_state(
        is_idle: Arc<Mutex<bool>>,
        last_activity: Arc<Mutex<Instant>>,
        timeout_ms: u32,
    ) -> Result<(), String> {
        // Connect to Wayland
        let mut conn =
            Connection::connect().map_err(|e| format!("Failed to connect to Wayland: {}", e))?;

        let mut state = IdleState {
            is_idle: Arc::clone(&is_idle),
            last_activity: Arc::clone(&last_activity),
            seats: Seats::new(&mut conn),
            seat_names: HashMap::default(),
        };

        // Receive seats
        conn.blocking_roundtrip()
            .map_err(|e| format!("Failed to receive seats: {}", e))?;
        conn.dispatch_events(&mut state);

        // Receive seat names
        conn.blocking_roundtrip()
            .map_err(|e| format!("Failed to receive seat names: {}", e))?;
        conn.dispatch_events(&mut state);

        // Get the first available seat
        let seat = state
            .seats
            .iter()
            .next()
            .ok_or_else(|| "No Wayland seats found".to_string())?;

        // Bind to idle notifier
        let idle_notifier = conn
            .bind_singleton::<ExtIdleNotifierV1>(1..=1)
            .map_err(|e| format!("Failed to bind idle notifier: {}", e))?;

        // Register idle notification
        idle_notifier.get_idle_notification_with_cb(
            &mut conn,
            timeout_ms,
            seat,
            idle_notification_cb,
        );

        // Main event loop - keep connection alive and process events
        loop {
            conn.flush(IoMode::Blocking)
                .map_err(|e| format!("Failed to flush: {}", e))?;

            conn.recv_events(IoMode::Blocking)
                .map_err(|e| format!("Failed to receive events: {}", e))?;

            conn.dispatch_events(&mut state);
        }
    }

    /// Check if currently idle
    pub fn is_idle(&self) -> Result<bool, String> {
        Ok(*self.is_idle.lock().unwrap())
    }

    /// Get idle time duration
    pub fn get_idle_time(&self) -> Result<Duration, String> {
        let is_idle = *self.is_idle.lock().unwrap();

        if is_idle {
            // Calculate how long we've been idle
            let last_activity = *self.last_activity.lock().unwrap();
            Ok(last_activity.elapsed())
        } else {
            Ok(Duration::from_secs(0))
        }
    }
}

struct IdleState {
    is_idle: Arc<Mutex<bool>>,
    last_activity: Arc<Mutex<Instant>>,
    seats: Seats,
    seat_names: HashMap<CString, WlSeat>,
}

impl SeatHandler for IdleState {
    fn get_seats(&mut self) -> &mut Seats {
        &mut self.seats
    }

    fn seat_name(&mut self, _: &mut Connection<Self>, wl_seat: WlSeat, name: CString) {
        self.seat_names.insert(name, wl_seat);
    }
}

fn idle_notification_cb(ctx: EventCtx<IdleState, ExtIdleNotificationV1>) {
    match ctx.event {
        ext_idle_notification_v1::Event::Idled => {
            // User became idle
            *ctx.state.is_idle.lock().unwrap() = true;
            *ctx.state.last_activity.lock().unwrap() = Instant::now();
        }
        ext_idle_notification_v1::Event::Resumed => {
            // User became active again
            *ctx.state.is_idle.lock().unwrap() = false;
            *ctx.state.last_activity.lock().unwrap() = Instant::now();
        }
        _ => {}
    }
}
