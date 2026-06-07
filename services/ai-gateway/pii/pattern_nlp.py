"""Pattern-only NLP engine for local recognizer tests.

Presidio's default AnalyzerEngine attempts to install/load a large spaCy model
on first use. FR-AI-012 recognizers are regex + lookup-table based, so tests use
this minimal engine to avoid network/model downloads while still exercising the
Presidio analyzer pipeline.
"""

from __future__ import annotations

from typing import Iterable, Iterator, Tuple

import spacy
from presidio_analyzer import AnalyzerEngine
from presidio_analyzer.nlp_engine import NlpArtifacts, NlpEngine


class PatternOnlyNlpEngine(NlpEngine):
    """Minimal NlpEngine backed by blank spaCy tokenizers."""

    engine_name = "pattern_only"

    def __init__(self):
        self.nlp = {
            "en": spacy.blank("en"),
            "vi": spacy.blank("xx"),
        }
        self._loaded = False

    def load(self) -> None:
        self._loaded = True

    def is_loaded(self) -> bool:
        return self._loaded

    def process_text(self, text: str, language: str) -> NlpArtifacts:
        doc = self.nlp.get(language, self.nlp["en"])(text)
        return NlpArtifacts(
            entities=[],
            tokens=doc,
            tokens_indices=[token.idx for token in doc],
            lemmas=[token.text for token in doc],
            nlp_engine=self,
            language=language,
        )

    def process_batch(
        self,
        texts: Iterable[str],
        language: str,
        batch_size: int = 1,
        n_process: int = 1,
        **kwargs,
    ) -> Iterator[Tuple[str, NlpArtifacts]]:
        del batch_size, n_process, kwargs
        for text in texts:
            yield text, self.process_text(text, language)

    def is_stopword(self, word: str, language: str) -> bool:
        return word.lower() in self.nlp.get(language, self.nlp["en"]).Defaults.stop_words

    def is_punct(self, word: str, language: str) -> bool:
        return all(ch in ".,;:!?()[]{}\"'" for ch in word)

    def get_supported_entities(self):
        return []

    def get_supported_languages(self):
        return ["en", "vi"]


def create_pattern_analyzer() -> AnalyzerEngine:
    engine = PatternOnlyNlpEngine()
    engine.load()
    return AnalyzerEngine(
        nlp_engine=engine,
        supported_languages=["en", "vi"],
        context_aware_enhancer=None,
    )
