from __future__ import annotations

from dataclasses import dataclass, field
from typing import AsyncIterator, Dict, List, Optional


class SDKError(Exception):
    pass


@dataclass
class _TaskRecord:
    creator: str
    task_type: str
    description: str
    assignee: Optional[str] = None
    accepted: bool = False


@dataclass
class _EscrowRecord:
    payer: str
    payee: str
    amount: int
    released: bool = False


@dataclass
class _AgentRecord:
    did: str
    metadata: Dict[str, object]


class KAMNClient:
    def __init__(self) -> None:
        self._agent_seq = 1
        self._msg_seq = 1
        self._task_seq = 1
        self._escrow_seq = 1
        self._agents: Dict[str, _AgentRecord] = {}
        self._inboxes: Dict[str, List[Dict[str, object]]] = {}
        self._tasks: Dict[str, _TaskRecord] = {}
        self._escrows: Dict[str, _EscrowRecord] = {}
        self._balances: Dict[str, int] = {}
        self._reputation: Dict[str, Dict[str, object]] = {}

    def register(
        self, agent_type: str, model_family: str, capabilities: List[str]
    ) -> str:
        if not agent_type.strip():
            raise SDKError("agent_type must not be empty")
        if not model_family.strip():
            raise SDKError("model_family must not be empty")
        if not capabilities:
            raise SDKError("capabilities must not be empty")
        if any(not capability.strip() for capability in capabilities):
            raise SDKError("capabilities must not include empty entries")

        did = f"kamn:did:agent:agent_{self._agent_seq}"
        self._agent_seq += 1
        metadata = {
            "agent_type": agent_type,
            "model_family": model_family,
            "capabilities": list(capabilities),
        }
        self._agents[did] = _AgentRecord(did=did, metadata=metadata)
        self._inboxes[did] = []
        self._balances[did] = 100
        self._reputation[did] = {"id": did, "score": 500}
        return did

    def resolve(self, did: str) -> Dict[str, object]:
        agent = self._agents.get(did)
        if agent is None:
            raise SDKError(f"unknown did: {did}")
        return {
            "id": agent.did,
            "metadata": dict(agent.metadata),
            "service_endpoint": f"kamn://messaging/{agent.did}",
        }

    def send(self, from_did: str, to_did: str, body: str) -> str:
        self._ensure_known_agent(from_did)
        self._ensure_known_agent(to_did)
        if not body.strip():
            raise SDKError("message body must not be empty")

        message_id = f"msg_{self._msg_seq}"
        self._msg_seq += 1
        self._inboxes[to_did].append(
            {"id": message_id, "from": from_did, "to": to_did, "body": body}
        )
        return message_id

    def receive(self, did: str) -> List[Dict[str, object]]:
        self._ensure_known_agent(did)
        inbox = self._inboxes[did]
        drained = list(inbox)
        inbox.clear()
        return drained

    async def receive_stream(self, did: str) -> AsyncIterator[Dict[str, object]]:
        for message in self.receive(did):
            yield dict(message)

    def create_task(self, creator_did: str, task_type: str, description: str) -> str:
        self._ensure_known_agent(creator_did)
        if not task_type.strip():
            raise SDKError("task_type must not be empty")
        if not description.strip():
            raise SDKError("description must not be empty")

        task_id = f"task_{self._task_seq}"
        self._task_seq += 1
        self._tasks[task_id] = _TaskRecord(
            creator=creator_did,
            task_type=task_type,
            description=description,
        )
        return task_id

    def accept_task(self, task_id: str, assignee_did: str) -> None:
        self._ensure_known_agent(assignee_did)
        task = self._tasks.get(task_id)
        if task is None:
            raise SDKError(f"unknown task: {task_id}")
        if task.accepted:
            raise SDKError("task already accepted")

        task.assignee = assignee_did
        task.accepted = True

    def create_escrow(self, payer_did: str, payee_did: str, amount: int) -> str:
        self._ensure_known_agent(payer_did)
        self._ensure_known_agent(payee_did)
        if amount <= 0:
            raise SDKError("escrow amount must be positive")
        payer_balance = self._balances[payer_did]
        if payer_balance < amount:
            raise SDKError("insufficient funds")

        escrow_id = f"escrow_{self._escrow_seq}"
        self._escrow_seq += 1
        self._balances[payer_did] -= amount
        self._escrows[escrow_id] = _EscrowRecord(
            payer=payer_did, payee=payee_did, amount=amount
        )
        return escrow_id

    def release_escrow(self, escrow_id: str) -> None:
        escrow = self._escrows.get(escrow_id)
        if escrow is None:
            raise SDKError(f"unknown escrow: {escrow_id}")
        if escrow.released:
            raise SDKError("escrow already released")

        self._balances[escrow.payee] += escrow.amount
        escrow.released = True

    def balance(self, did: str) -> int:
        self._ensure_known_agent(did)
        return self._balances[did]

    def search_agents(
        self, capability: Optional[str] = None, model_family: Optional[str] = None
    ) -> List[Dict[str, object]]:
        results: List[Dict[str, object]] = []
        for agent in self._agents.values():
            metadata = agent.metadata
            if model_family and metadata["model_family"] != model_family:
                continue
            if capability and capability not in metadata["capabilities"]:
                continue
            results.append({"id": agent.did, "metadata": dict(metadata)})
        return sorted(results, key=lambda item: str(item["id"]))

    def get_reputation(self, did: str) -> Dict[str, object]:
        self._ensure_known_agent(did)
        return dict(self._reputation[did])

    def _ensure_known_agent(self, did: str) -> None:
        if did not in self._agents:
            raise SDKError(f"unknown did: {did}")
