"""End-to-end tests for `SocketError` bindings exposed via PyO3."""

from __future__ import annotations

import pytest

from barter_python import SocketError, SocketErrorInfo, _testing_raise_socket_error


def test_socket_error_exposes_kind_message_and_details() -> None:
    with pytest.raises(SocketError) as excinfo:
        _testing_raise_socket_error("subscribe")

    err = excinfo.value
    assert err.kind == "Subscribe"
    assert "error subscribing" in err.message

    details = err.details
    assert isinstance(details, dict)
    assert details["message"] == "subscription failed"

    info = err.info
    assert isinstance(info, SocketErrorInfo)
    assert info.kind == err.kind

    info_details = info.details
    assert isinstance(info_details, dict)
    assert info_details["message"] == "subscription failed"


def test_socket_error_binary_payload_round_trip() -> None:
    with pytest.raises(SocketError) as excinfo:
        _testing_raise_socket_error("deserialise_binary")

    err = excinfo.value
    assert err.kind == "DeserialiseBinary"

    details = err.details
    assert isinstance(details, dict)
    assert details["payload"] == b"\x01\x02\x03"

    info = err.info
    assert isinstance(info, SocketErrorInfo)
    info_details = info.details
    assert isinstance(info_details, dict)
    assert info_details["payload"] == b"\x01\x02\x03"
