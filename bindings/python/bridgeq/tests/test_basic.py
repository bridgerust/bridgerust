import pytest

from bridgeq import BridgeQueue


def test_enqueue_dequeue_ack_flow():
    queue = BridgeQueue(max_retries=2)
    first = queue.enqueue("job-1")
    queue.enqueue_batch(["job-2", "job-3"])

    messages = queue.dequeue(2)
    assert len(messages) == 2
    assert messages[0].id == first
    assert messages[0].payload == "job-1"
    assert messages[0].attempts == 0

    assert queue.ack(messages[0].id) is True
    assert queue.ack(messages[0].id) is False


def test_nack_dead_letter_and_requeue():
    queue = BridgeQueue(max_retries=0)
    item_id = queue.enqueue("job-fail")
    item = queue.dequeue(1)[0]
    assert item.id == item_id
    assert queue.nack(item_id) is True

    stats = queue.stats()
    assert stats.dead_letter == 1
    assert stats.ready == 0
    assert stats.in_flight == 0

    assert queue.requeue_dead_letter(item_id) is True
    assert queue.stats().ready == 1


def test_memory_adapter_selection():
    queue = BridgeQueue(adapter="memory", max_retries=1, retry_backoff_ms=10)
    item_id = queue.enqueue("job-adapter")
    item = queue.dequeue(1)[0]
    assert item.id == item_id
    assert queue.ack(item_id) is True


def test_invalid_adapter_raises():
    with pytest.raises(ValueError):
        BridgeQueue(adapter="unknown")
