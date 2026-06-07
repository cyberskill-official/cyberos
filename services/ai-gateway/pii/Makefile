# FR-AI-013 — Makefile for VN PII recall gate and related targets.

.PHONY: pii-recall-test pii-fixture-validate pii-precision-report pii-all pii-no-false-positives pii-network-lint

PYTHON ?= python3
PYTEST ?= $(PYTHON) -m pytest

pii-all: pii-fixture-validate pii-recall-test pii-precision-report pii-no-false-positives pii-network-lint

pii-recall-test:
	CYBEROS_PII_PATTERN_ONLY_NLP=1 $(PYTEST) \
		tests/test_recall_gate.py \
		tests/test_fixture_invariants.py \
		tests/test_no_real_pii_in_corpus.py \
		tests/test_recall_gate_runtime_budget.py \
		-v --tb=short

pii-fixture-validate:
	$(PYTHON) scripts/validate_corpus_format.py

pii-precision-report:
	CYBEROS_PII_PATTERN_ONLY_NLP=1 $(PYTEST) tests/test_precision_warning.py -v --tb=short

pii-no-false-positives:
	CYBEROS_PII_PATTERN_ONLY_NLP=1 $(PYTEST) test_vn_no_false_positives.py -v --tb=short

pii-network-lint:
	$(PYTEST) test_no_network_imports.py -v --tb=short
