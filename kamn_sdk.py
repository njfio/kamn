from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import (
    AsyncIterator,
    Callable,
    Dict,
    List,
    Literal,
    Optional,
    Protocol,
    TypedDict,
    TypeVar,
)


class SDKError(Exception):
    pass


class TransportMode(str, Enum):
    IN_MEMORY = "in-memory"
    LIVE = "live"


class TransportModeMismatchError(SDKError):
    def __init__(self, expected: TransportMode, found: TransportMode) -> None:
        self.expected = expected.value
        self.found = found.value
        super().__init__(
            f"transport mode mismatch, expected {self.expected}, found {self.found}"
        )


@dataclass(frozen=True)
class LiveTransportConfig:
    endpoint: str

    def __post_init__(self) -> None:
        normalized = self.endpoint.strip().lower()
        if not (normalized.startswith("https://") or normalized.startswith("wss://")):
            raise SDKError("transport endpoint must start with https:// or wss://")
        if len(normalized) <= len("https://a"):
            raise SDKError("transport endpoint must include host information")


LiveTransportOperation = Literal[
    "register",
    "resolve",
    "send",
    "receive",
    "createTask",
    "acceptTask",
    "createEscrow",
    "releaseEscrow",
    "balance",
    "searchAgents",
    "getReputation",
]


class LiveTransportBackendRequest(TypedDict):
    endpoint: str
    operation: LiveTransportOperation
    payload: Dict[str, object]


class LiveTransportBackendResponseOk(TypedDict):
    status: Literal["ok"]
    value: object


class LiveTransportBackendResponseError(TypedDict):
    status: Literal["error"]
    reason: str


LiveTransportBackendResponse = (
    LiveTransportBackendResponseOk | LiveTransportBackendResponseError
)


class LiveTransportBackendAdapter(Protocol):
    def invoke(
        self, request: LiveTransportBackendRequest
    ) -> LiveTransportBackendResponse: ...


class LiveTransportBackendAdapterError(SDKError):
    def __init__(self, operation: LiveTransportOperation, reason: str) -> None:
        self.operation = operation
        self.reason = reason
        super().__init__(f"backend adapter operation {operation} failed: {reason}")


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

    def transport_mode(self) -> TransportMode:
        return TransportMode.IN_MEMORY

    def assert_transport_mode(self, expected: TransportMode) -> None:
        found = self.transport_mode()
        if found != expected:
            raise TransportModeMismatchError(expected, found)

    def _ensure_known_agent(self, did: str) -> None:
        if did not in self._agents:
            raise SDKError(f"unknown did: {did}")


_T = TypeVar("_T")


class LiveKAMNClient:
    _live_endpoints: Dict[str, KAMNClient] = {}
    _backend_adapters: Dict[str, LiveTransportBackendAdapter] = {}

    @classmethod
    def register_backend_adapter(
        cls, endpoint: str, adapter: LiveTransportBackendAdapter
    ) -> None:
        config = LiveTransportConfig(endpoint=endpoint)
        cls._backend_adapters[config.endpoint] = adapter

    @classmethod
    def clear_backend_adapters(cls) -> None:
        cls._backend_adapters.clear()

    def __init__(self, endpoint: str) -> None:
        self.config = LiveTransportConfig(endpoint=endpoint)
        self._endpoint = self.config.endpoint
        if self._endpoint not in self._live_endpoints:
            self._live_endpoints[self._endpoint] = KAMNClient()
        self._delegate = self._live_endpoints[self._endpoint]
        self._backend_adapter = self._backend_adapters.get(self._endpoint)

    @property
    def endpoint(self) -> str:
        return self._endpoint

    def transport_mode(self) -> TransportMode:
        return TransportMode.LIVE

    def assert_transport_mode(self, expected: TransportMode) -> None:
        found = self.transport_mode()
        if found != expected:
            raise TransportModeMismatchError(expected, found)

    def register(
        self, agent_type: str, model_family: str, capabilities: List[str]
    ) -> str:
        return self._invoke_with_adapter(
            "register",
            {
                "agentType": agent_type,
                "modelFamily": model_family,
                "capabilities": list(capabilities),
            },
            lambda value: self._require_string_value("register", value),
            lambda: self._delegate.register(agent_type, model_family, capabilities),
        )

    def resolve(self, did: str) -> Dict[str, object]:
        return self._invoke_with_adapter(
            "resolve",
            {"did": did},
            lambda value: self._require_resolve_value("resolve", value),
            lambda: self._delegate.resolve(did),
        )

    def send(self, from_did: str, to_did: str, body: str) -> str:
        return self._invoke_with_adapter(
            "send",
            {"fromDid": from_did, "toDid": to_did, "body": body},
            lambda value: self._require_string_value("send", value),
            lambda: self._delegate.send(from_did, to_did, body),
        )

    def receive(self, did: str) -> List[Dict[str, object]]:
        return self._invoke_with_adapter(
            "receive",
            {"did": did},
            lambda value: self._require_inbox_messages_value("receive", value),
            lambda: self._delegate.receive(did),
        )

    async def receive_stream(self, did: str) -> AsyncIterator[Dict[str, object]]:
        for message in self.receive(did):
            yield dict(message)

    def create_task(self, creator_did: str, task_type: str, description: str) -> str:
        return self._invoke_with_adapter(
            "createTask",
            {"creatorDid": creator_did, "taskType": task_type, "description": description},
            lambda value: self._require_string_value("createTask", value),
            lambda: self._delegate.create_task(creator_did, task_type, description),
        )

    def accept_task(self, task_id: str, assignee_did: str) -> None:
        self._invoke_with_adapter(
            "acceptTask",
            {"taskId": task_id, "assigneeDid": assignee_did},
            lambda _: None,
            lambda: self._delegate.accept_task(task_id, assignee_did),
        )

    def create_escrow(self, payer_did: str, payee_did: str, amount: int) -> str:
        return self._invoke_with_adapter(
            "createEscrow",
            {"payerDid": payer_did, "payeeDid": payee_did, "amount": amount},
            lambda value: self._require_string_value("createEscrow", value),
            lambda: self._delegate.create_escrow(payer_did, payee_did, amount),
        )

    def release_escrow(self, escrow_id: str) -> None:
        self._invoke_with_adapter(
            "releaseEscrow",
            {"escrowId": escrow_id},
            lambda _: None,
            lambda: self._delegate.release_escrow(escrow_id),
        )

    def balance(self, did: str) -> int:
        return self._invoke_with_adapter(
            "balance",
            {"did": did},
            lambda value: self._require_int_value("balance", value),
            lambda: self._delegate.balance(did),
        )

    def search_agents(
        self, capability: Optional[str] = None, model_family: Optional[str] = None
    ) -> List[Dict[str, object]]:
        payload: Dict[str, object] = {}
        if capability is not None:
            payload["capability"] = capability
        if model_family is not None:
            payload["modelFamily"] = model_family
        return self._invoke_with_adapter(
            "searchAgents",
            payload,
            lambda value: self._require_search_agents_value("searchAgents", value),
            lambda: self._delegate.search_agents(capability=capability, model_family=model_family),
        )

    def get_reputation(self, did: str) -> Dict[str, object]:
        return self._invoke_with_adapter(
            "getReputation",
            {"did": did},
            lambda value: self._require_reputation_value("getReputation", value),
            lambda: self._delegate.get_reputation(did),
        )

    def __getattr__(self, name: str):
        return getattr(self._delegate, name)

    def _invoke_with_adapter(
        self,
        operation: LiveTransportOperation,
        payload: Dict[str, object],
        normalize: Callable[[object], _T],
        fallback: Callable[[], _T],
    ) -> _T:
        if self._backend_adapter is None:
            return fallback()

        response = self._backend_adapter.invoke(
            {"endpoint": self._endpoint, "operation": operation, "payload": payload}
        )
        if not isinstance(response, dict):
            self._raise_invalid_adapter_response(operation, "expected mapping response")

        status = response.get("status")
        if status == "error":
            reason_raw = response.get("reason")
            reason = (
                reason_raw.strip()
                if isinstance(reason_raw, str) and reason_raw.strip()
                else "backend adapter returned unknown error"
            )
            raise LiveTransportBackendAdapterError(operation, reason)

        if status != "ok":
            self._raise_invalid_adapter_response(operation, "expected status ok|error")
        return normalize(response.get("value"))

    def _raise_invalid_adapter_response(
        self, operation: LiveTransportOperation, reason: str
    ) -> None:
        raise SDKError(
            f"backend adapter invalid response for operation {operation}: {reason}"
        )

    def _require_string_value(
        self, operation: LiveTransportOperation, value: object
    ) -> str:
        if not isinstance(value, str) or not value.strip():
            self._raise_invalid_adapter_response(operation, "expected string value")
        return value

    def _require_int_value(self, operation: LiveTransportOperation, value: object) -> int:
        if not isinstance(value, int) or isinstance(value, bool):
            self._raise_invalid_adapter_response(operation, "expected integer value")
        return value

    def _require_resolve_value(
        self, operation: LiveTransportOperation, value: object
    ) -> Dict[str, object]:
        if not isinstance(value, dict):
            self._raise_invalid_adapter_response(operation, "expected resolve record")

        agent_id = value.get("id")
        service_endpoint = value.get("service_endpoint")
        metadata = value.get("metadata", {})
        if not isinstance(agent_id, str) or not isinstance(service_endpoint, str):
            self._raise_invalid_adapter_response(operation, "expected resolve record")
        if not isinstance(metadata, dict):
            self._raise_invalid_adapter_response(operation, "expected resolve metadata map")
        return {
            "id": agent_id,
            "metadata": dict(metadata),
            "service_endpoint": service_endpoint,
        }

    def _require_inbox_messages_value(
        self, operation: LiveTransportOperation, value: object
    ) -> List[Dict[str, object]]:
        if not isinstance(value, list):
            self._raise_invalid_adapter_response(operation, "expected inbox message array")
        normalized: List[Dict[str, object]] = []
        for entry in value:
            if not isinstance(entry, dict):
                self._raise_invalid_adapter_response(
                    operation, "expected inbox message object"
                )

            message_id = entry.get("id")
            from_did = entry.get("from")
            to_did = entry.get("to")
            body = entry.get("body")
            if not isinstance(message_id, str):
                self._raise_invalid_adapter_response(
                    operation, "expected inbox message string fields"
                )
            if not isinstance(from_did, str):
                self._raise_invalid_adapter_response(
                    operation, "expected inbox message string fields"
                )
            if not isinstance(to_did, str):
                self._raise_invalid_adapter_response(
                    operation, "expected inbox message string fields"
                )
            if not isinstance(body, str):
                self._raise_invalid_adapter_response(
                    operation, "expected inbox message string fields"
                )

            envelope = entry.get("envelope")
            if envelope is None:
                envelope = {"id": message_id}
            normalized.append(
                {
                    "id": message_id,
                    "from": from_did,
                    "to": to_did,
                    "body": body,
                    "envelope": envelope,
                }
            )
        return normalized

    def _require_search_agents_value(
        self, operation: LiveTransportOperation, value: object
    ) -> List[Dict[str, object]]:
        if not isinstance(value, list):
            self._raise_invalid_adapter_response(
                operation, "expected search agent result array"
            )

        normalized: List[Dict[str, object]] = []
        for entry in value:
            if not isinstance(entry, dict):
                self._raise_invalid_adapter_response(
                    operation, "expected search agent result object"
                )
            agent_id = entry.get("id")
            metadata = entry.get("metadata", {})
            if not isinstance(agent_id, str) or not isinstance(metadata, dict):
                self._raise_invalid_adapter_response(
                    operation, "expected search agent result object"
                )
            normalized.append({"id": agent_id, "metadata": dict(metadata)})
        return normalized

    def _require_reputation_value(
        self, operation: LiveTransportOperation, value: object
    ) -> Dict[str, object]:
        if not isinstance(value, dict):
            self._raise_invalid_adapter_response(operation, "expected reputation record")

        agent_id = value.get("id")
        score = value.get("score")
        if not isinstance(agent_id, str):
            self._raise_invalid_adapter_response(operation, "expected reputation record")
        if not isinstance(score, (int, float)) or isinstance(score, bool):
            self._raise_invalid_adapter_response(operation, "expected reputation record")
        return {"id": agent_id, "score": score}
