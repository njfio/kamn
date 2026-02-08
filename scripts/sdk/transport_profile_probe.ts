import {
  KAMNClient,
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

  console.log("status=ok");
  console.log(`default_transport_mode=${memory.transportMode()}`);
  console.log(`live_transport_mode=${live.transportMode()}`);
  console.log(`memory_mismatch_expected=${memoryExpected}`);
  console.log(`memory_mismatch_found=${memoryFound}`);
  console.log(`live_mismatch_expected=${liveExpected}`);
  console.log(`live_mismatch_found=${liveFound}`);
} catch (error: unknown) {
  if (error instanceof SDKError || error instanceof Error) {
    fail(error.message);
  }
  fail("unknown error");
}
