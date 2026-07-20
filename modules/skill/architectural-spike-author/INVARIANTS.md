# architectural-spike-author - invariants

INV-1 timebox_hours is recorded before any probing; a spike without it is invalid. INV-2 every evidence entry is checkable (repo path, command+output, or URL) - uncited
      assertions count as zero evidence.
INV-3 the recommendation names exactly one PROBED option. INV-4 the discard log is non-empty whenever any option was rejected. INV-5 actual_hours > 1.5x timebox_hours forces a HALT with a recorded operator verdict. INV-6 artefact version is pinned at architectural-spike@1; changes go through the
      contracts CHANGELOG discipline.
INV-7 one spike = one question; multi-question requests split into multiple spikes.
