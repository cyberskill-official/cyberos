# FR-AI-013 — Makefile for VN PII recall gate and related targets.

.PHONY: pii-recall-test pii-fixture-validate pii-precision-report pii-all

pii-all: pii-fixture-validate pii-recall-test pii-precision-report

pii-recall-test:
	pytest test_vn_recall_floor.py test_vn_recognizers.py -v --tb=short

pii-fixture-validate:
	python scripts/validate_corpus_format.py

pii-precision-report:
	@echo "Precision reporting not yet implemented (FR-AI-022)."
	@echo "Run: pytest test_vn_recognizers.py -v for current status."

pii-no-false-positives:
	pytest test_vn_no_false_positives.py -v --tb=short

pii-network-lint:
	pytest test_no_network_imports.py -v
