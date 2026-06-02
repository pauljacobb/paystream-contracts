# Monitoring and Uptime Alerting

This document defines the external uptime monitoring strategy for PayStream and the required alerting configuration for production.

## External uptime checks

- Perform uptime checks every 60 seconds.
- Run checks from at least three separate regions to verify global availability.
- Use a public health or probe endpoint, for example `https://api.paystream.example/health`.
- Alert only after two consecutive failures to reduce noise from transient network blips.
- Ensure each regional probe is independent and has separate DNS/HTTP routing.

## Alerting integration

- Integrate monitoring with a paging system such as PagerDuty or OpsGenie.
- Configure a dedicated incident service or escalation policy for PayStream uptime outages.
- Alert on a sustained outage condition defined as two consecutive probe failures.
- Include clear incident metadata and remediation guidance in alert messages.

## On-call rotation

- Configure an on-call schedule in PagerDuty/OpsGenie for PayStream production operations.
- Include a primary responder and escalation path.
- Verify the schedule is documented in the team runbook and updated whenever team membership changes.
- Ensure the on-call schedule is linked from the incident response runbook.

## Status page

- Publish a public status page at `status.paystream.example`.
- The status page should reflect the external uptime check status and the current incident state.
- Ensure scheduled maintenance and active incident notices are visible.
- Link the status page from customer-facing communications and internal service documentation.

## Recommended production setup

1. Create an uptime monitor for the public API endpoint.
2. Configure three geographic check locations.
3. Set the check interval to 60 seconds.
4. Require two consecutive failures before triggering an alert.
5. Bind the monitor to the PagerDuty/OpsGenie incident service.
6. Create or update the status page for PayStream availability.

## Notes

- This monitoring must be external to the PayStream infrastructure; it should not depend on internal metrics alone.
- If the product exposes additional user-facing endpoints or web UI, add those as secondary synthesis checks after the primary API monitor is validated.
- Keep alert thresholds and runbook links consistent across PagerDuty/OpsGenie, monitoring provider, and incident response documents.
