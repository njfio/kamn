import {
  KAMNClient,
  LiveTransportBackendAdapterError,
  LiveTransportKAMNClient,
  SDKError,
  TransportModeMismatchError,
} from "../../packages/kamn-sdk/src/index.ts";

function sanitize(value: string): string {
  return value.replaceAll("\n", " ");
}

function fail(message: string): never {
  console.log("status=error");
  console.log(`error=${sanitize(message)}`);
  process.exit(1);
}

try {
  const memory = new KAMNClient();
  const live = new LiveTransportKAMNClient("https://live.kamn.testnet/profile-probe-ts");

  let memoryExpected = "";
  let memoryFound = "";
  try {
    memory.assertTransportMode("live");
    fail("memory client unexpectedly accepted live mode assertion");
  } catch (error: unknown) {
    if (error instanceof TransportModeMismatchError) {
      memoryExpected = error.expected;
      memoryFound = error.found;
    } else if (error instanceof Error) {
      fail(error.message);
    } else {
      fail("unknown memory assertion error");
    }
  }

  let liveExpected = "";
  let liveFound = "";
  try {
    live.assertTransportMode("in-memory");
    fail("live client unexpectedly accepted in-memory mode assertion");
  } catch (error: unknown) {
    if (error instanceof TransportModeMismatchError) {
      liveExpected = error.expected;
      liveFound = error.found;
    } else if (error instanceof Error) {
      fail(error.message);
    } else {
      fail("unknown live assertion error");
    }
  }

  const successEndpoint = "https://live.kamn.testnet/profile-probe-ts-adapter";
  LiveTransportKAMNClient.registerBackendAdapter(successEndpoint, {
    invoke(request) {
      if (request.operation === "register") {
        return { status: "ok", value: "kamn:did:agent:backend-1" };
      }
      if (request.operation === "send") {
        return { status: "ok", value: "msg_backend_1" };
      }
      if (request.operation === "receive") {
        return {
          status: "ok",
          value: [
            {
              id: "msg_backend_1",
              from: "kamn:did:agent:backend-1",
              to: "kamn:did:agent:backend-2",
              body: "backend hello",
            },
          ],
        };
      }
      return { status: "ok", value: null };
    },
  });

  let backendAdapterRegisterId = "";
  let backendAdapterMessageId = "";
  let backendAdapterReceiveBody = "";
  try {
    const adapterClient = new LiveTransportKAMNClient(successEndpoint);
    backendAdapterRegisterId = adapterClient.register("autonomous", "claude-4", ["text"]);
    backendAdapterMessageId = adapterClient.send(
      "kamn:did:agent:backend-1",
      "kamn:did:agent:backend-2",
      "backend hello",
    );
    const received = adapterClient.receive("kamn:did:agent:backend-2");
    if (received.length !== 1) {
      fail("backend adapter receive returned unexpected message count");
    }
    backendAdapterReceiveBody = received[0].body;
  } finally {
    LiveTransportKAMNClient.clearBackendAdapters();
  }

  const failureEndpoint = "https://live.kamn.testnet/profile-probe-ts-adapter-fail";
  LiveTransportKAMNClient.registerBackendAdapter(failureEndpoint, {
    invoke(request) {
      if (request.operation === "register") {
        return { status: "ok", value: 7 };
      }
      if (request.operation === "send") {
        return { status: "error", reason: "backend_timeout" };
      }
      return { status: "error", reason: "policy_denied" };
    },
  });

  let backendAdapterInvalidResponseMessage = "";
  let backendAdapterErrorOperation = "";
  let backendAdapterErrorReason = "";
  let backendAdapterPolicyReason = "";
  try {
    const failingClient = new LiveTransportKAMNClient(failureEndpoint);
    try {
      failingClient.register("autonomous", "claude-4", ["text"]);
      fail("failing adapter unexpectedly accepted invalid register payload");
    } catch (error: unknown) {
      if (error instanceof SDKError || error instanceof Error) {
        backendAdapterInvalidResponseMessage = error.message;
      } else {
        fail("unknown adapter register failure");
      }
    }

    try {
      failingClient.send("kamn:did:agent:x", "kamn:did:agent:y", "hello");
      fail("failing adapter unexpectedly accepted send operation");
    } catch (error: unknown) {
      if (error instanceof LiveTransportBackendAdapterError) {
        backendAdapterErrorOperation = error.operation;
        backendAdapterErrorReason = error.reason;
      } else if (error instanceof Error) {
        fail(error.message);
      } else {
        fail("unknown adapter send failure");
      }
    }

    try {
      failingClient.receive("kamn:did:agent:y");
      fail("failing adapter unexpectedly accepted receive operation");
    } catch (error: unknown) {
      if (error instanceof LiveTransportBackendAdapterError) {
        backendAdapterPolicyReason = error.reason;
      } else if (error instanceof Error) {
        fail(error.message);
      } else {
        fail("unknown adapter receive failure");
      }
    }
  } finally {
    LiveTransportKAMNClient.clearBackendAdapters();
  }

  console.log("status=ok");
  console.log(`default_transport_mode=${memory.transportMode()}`);
  console.log(`live_transport_mode=${live.transportMode()}`);
  console.log(`memory_mismatch_expected=${memoryExpected}`);
  console.log(`memory_mismatch_found=${memoryFound}`);
  console.log(`live_mismatch_expected=${liveExpected}`);
  console.log(`live_mismatch_found=${liveFound}`);
  console.log(`backend_adapter_register_id=${backendAdapterRegisterId}`);
  console.log(`backend_adapter_message_id=${backendAdapterMessageId}`);
  console.log(`backend_adapter_receive_body=${backendAdapterReceiveBody}`);
  console.log(
    `backend_adapter_invalid_response_message=${sanitize(backendAdapterInvalidResponseMessage)}`,
  );
  console.log(`backend_adapter_error_operation=${backendAdapterErrorOperation}`);
  console.log(`backend_adapter_error_reason=${backendAdapterErrorReason}`);
  console.log(`backend_adapter_policy_reason=${backendAdapterPolicyReason}`);
} catch (error: unknown) {
  if (error instanceof SDKError || error instanceof Error) {
    fail(error.message);
  }
  fail("unknown error");
}
