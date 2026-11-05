fn main() {
    let request = sc_cron_commands::ListRequest {
        state: sc_cron_commands::ScheduleState::Active,
        limit: 100,
        cursor: None,
    };
    std::hint::black_box(request);
}
