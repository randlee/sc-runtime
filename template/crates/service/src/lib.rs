include!("layouts/{{db}}.rs");

{% if cron_commands %}
pub mod cron_commands;
{% endif %}
