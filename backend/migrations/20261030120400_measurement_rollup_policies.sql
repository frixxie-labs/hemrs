-- Added after the backfill so a policy run cannot race the initial refresh.
SELECT add_continuous_aggregate_policy(
    'measurements_hourly',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '30 minutes'
);

SELECT add_continuous_aggregate_policy(
    'measurements_daily',
    start_offset => INTERVAL '7 days',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour'
);
