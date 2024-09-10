
// Helper function to format duration
pub(crate) fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        format!("{}m", minutes)
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        format!("{}h", hours)
    } else {
        let days = seconds / 86400;
        format!("{}d", days)
    }
}

