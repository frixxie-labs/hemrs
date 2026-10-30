SELECT count(*) AS num_lightnings
FROM lightning_strikes
WHERE $__timeFilter(ts);
