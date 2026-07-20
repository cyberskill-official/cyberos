# TASK-IMP-095 observability injection

This task IS an observability injection: the deliverable is one diagnostic line at the exact moment an operator's edit is displaced. There is no consumer-runtime component beyond that line - so the honest record is what the line covers and what watches the line, not invented spans for a `cmp`.

- **Signal design**: fires only on information loss (prior file existed AND regeneration changed it); silent on fresh and identical paths so it cannot become noise that trains operators to ignore it. Names the two things the operator needs: the exact `.bak` path (recovery) and `.cyberos/config.yaml` (the durable fix).
- **The suite is the monitor**: t08 asserts all three arms every run - notice exactly once with a real, edit-carrying .bak on the edited path; zero lines on the two silent paths.
- **Failure visibility**: if the churn guard or bak naming ever drifts, t08's `-f "$bak"` + content grep fail loudly rather than the message pointing at a ghost file.

Branch coverage: 3 of 3 arms of the new conditional asserted (fresh, identical, differs) - 100 % of the change's executable surface.
