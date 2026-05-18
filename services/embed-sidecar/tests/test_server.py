"""Tests for the FR-AI-019 embedding sidecar (mock backend only — `real`
backend tests live behind a `--ignored` flag because they need GPU+weights)."""

from __future__ import annotations

import os

import pytest
from fastapi.testclient import TestClient

from cyberos_embed_sidecar.server import EMBED_DIM, MODEL_NAME, _mock_embed, app, embed_texts


@pytest.fixture(autouse=True)
def force_mock_mode(monkeypatch):
    monkeypatch.setenv("CYBEROS_EMBED_MODE", "mock")


@pytest.fixture
def client():
    return TestClient(app)


def test_mock_embed_returns_1024_floats() -> None:
    v = _mock_embed("hello world")
    assert len(v) == EMBED_DIM
    assert all(isinstance(x, float) for x in v)


def test_mock_embed_is_deterministic() -> None:
    a = _mock_embed("hello world")
    b = _mock_embed("hello world")
    assert a == b


def test_mock_embed_differentiates_inputs() -> None:
    a = _mock_embed("hello")
    b = _mock_embed("world")
    assert a != b


def test_mock_embed_is_unit_norm() -> None:
    v = _mock_embed("anything")
    norm_sq = sum(x * x for x in v)
    assert abs(norm_sq - 1.0) < 1e-5


def test_embed_texts_batch_mock() -> None:
    out = embed_texts(["foo", "bar", "baz"])
    assert len(out) == 3
    assert len(out[0]) == EMBED_DIM


def test_healthz_returns_metadata(client) -> None:
    resp = client.get("/healthz")
    assert resp.status_code == 200
    body = resp.json()
    assert body["model"] == MODEL_NAME
    assert body["dim"] == EMBED_DIM
    assert body["mode"] == "mock"


def test_embed_endpoint_round_trips(client) -> None:
    resp = client.post("/embed", json={"texts": ["alpha", "beta"], "model": "bge-m3"})
    assert resp.status_code == 200
    body = resp.json()
    assert body["model"] == MODEL_NAME
    assert body["dim"] == EMBED_DIM
    assert len(body["embeddings"]) == 2
    assert len(body["embeddings"][0]) == EMBED_DIM


def test_embed_endpoint_rejects_wrong_model(client) -> None:
    resp = client.post("/embed", json={"texts": ["x"], "model": "openai-ada-002"})
    assert resp.status_code == 400


def test_embed_endpoint_rejects_empty_batch(client) -> None:
    resp = client.post("/embed", json={"texts": []})
    # FastAPI/Pydantic validation surfaces this as 422
    assert resp.status_code == 422


def test_embed_endpoint_handles_default_model_field(client) -> None:
    resp = client.post("/embed", json={"texts": ["x"]})
    assert resp.status_code == 200
    assert resp.json()["model"] == MODEL_NAME
