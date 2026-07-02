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


# ---------------------------------------------------------------------------
# OpenAI-compatible /v1/embeddings — the shape the ai-gateway's local_openai
# provider builds (`{model, input}`) and parses (`data[].embedding` + usage).
# ---------------------------------------------------------------------------


def test_openai_embeddings_list_input(client) -> None:
    resp = client.post("/v1/embeddings", json={"model": "bge-m3", "input": ["alpha", "beta"]})
    assert resp.status_code == 200
    body = resp.json()
    assert body["object"] == "list"
    assert body["model"] == MODEL_NAME
    assert len(body["data"]) == 2
    assert [d["index"] for d in body["data"]] == [0, 1]
    for d in body["data"]:
        assert d["object"] == "embedding"
        assert len(d["embedding"]) == EMBED_DIM
        assert all(isinstance(x, float) for x in d["embedding"])
    assert body["usage"]["prompt_tokens"] >= 1
    assert body["usage"]["total_tokens"] == body["usage"]["prompt_tokens"]


def test_openai_embeddings_string_input_is_batch_of_one(client) -> None:
    resp = client.post("/v1/embeddings", json={"input": "just one text"})
    assert resp.status_code == 200
    body = resp.json()
    assert len(body["data"]) == 1
    assert len(body["data"][0]["embedding"]) == EMBED_DIM


def test_openai_embeddings_matches_native_route(client) -> None:
    """Both wire shapes must expose the same vectors for the same input."""
    native = client.post("/embed", json={"texts": ["cùng một văn bản"]}).json()
    compat = client.post("/v1/embeddings", json={"input": ["cùng một văn bản"]}).json()
    assert compat["data"][0]["embedding"] == native["embeddings"][0]


def test_openai_embeddings_rejects_wrong_model(client) -> None:
    resp = client.post("/v1/embeddings", json={"model": "text-embedding-3-small", "input": ["x"]})
    assert resp.status_code == 400


def test_openai_embeddings_rejects_empty_list(client) -> None:
    resp = client.post("/v1/embeddings", json={"input": []})
    assert resp.status_code == 400


def test_healthz_reports_warm_state(client) -> None:
    body = client.get("/healthz").json()
    # Mock mode has nothing to load, so it always reports ready.
    assert body["warm"] == "ready"
