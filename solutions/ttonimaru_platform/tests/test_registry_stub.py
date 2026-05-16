from __future__ import annotations

from conftest import make_client


def test_registry_package_stub() -> None:
    client = make_client()
    res = client.get("/api/v1/registry/packages/표준/sample")
    assert res.status_code == 200
    body = res.json()
    assert body == {
        "schema": "ddn.ttonimaru.package_metadata_stub.v1",
        "scope": "표준",
        "name": "sample",
        "version": "latest",
        "status": "stub",
        "install_supported": False,
        "update_supported": False,
        "remove_supported": False,
        "public_registry_final": False,
    }


def test_registry_package_version_stub() -> None:
    client = make_client()
    res = client.get("/api/v1/registry/packages/표준/sample/1.0.0")
    assert res.status_code == 200
    body = res.json()
    assert body["version"] == "1.0.0"
    assert body["install_supported"] is False
    assert body["public_registry_final"] is False
