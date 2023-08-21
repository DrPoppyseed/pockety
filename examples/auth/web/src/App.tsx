import { Route, Routes } from "@solidjs/router";
import axios from "axios";
import {
  createSignal,
  Match,
  onMount,
  Show,
  Switch,
  Component,
} from "solid-js";

const FetchStatus = {
  Idle: "idle",
  Fetching: "fetching",
  Success: "success",
  Error: "error",
} as const;
type FetchStatusType = typeof FetchStatus[keyof typeof FetchStatus];

const Comp: Component = () => {
  const [authUri, setAuthUri] = createSignal<String | null>(null);

  const onLoginClick = async () => {
    const result = await axios.post("http://localhost:8080/auth/pocket");

    if (result.status === 200) {
      localStorage.setItem("token", result.data.requestToken);
      setAuthUri(result.data.authUri);
    }
  };

  return (
    <div
      style={{
        display: "flex",
        "flex-direction": "column",
        "align-items": "center",
      }}
    >
      <h1>Auth Example</h1>
      <button onClick={() => onLoginClick()}>Login with Pocket</button>
      <Show when={authUri()}>
        <a
          style={{
            "margin-top": "1rem",
          }}
          href={authUri().toString()}
        >
          Grant permissions for Pocket
        </a>
      </Show>
    </div>
  );
};

const Auth: Component = () => {
  const [token, setToken] = createSignal<string | null>(null);
  const [fetchStatus, setFetchStatus] = createSignal<FetchStatusType>(
    FetchStatus.Idle
  );

  onMount(async () => {
    setFetchStatus(FetchStatus.Fetching);

    const requestToken = localStorage.getItem("token");
    setToken(requestToken)
    const result = await axios.post("http://localhost:8080/auth/authorize", {
      requestToken
    });

    if (result.status === 200) {
      setFetchStatus(FetchStatus.Success);
    } else {
      setFetchStatus(FetchStatus.Error);
    }
  });

  return (
    <Switch fallback={<h1>Error</h1>}>
      <Match when={fetchStatus() === FetchStatus.Idle}>
        <h1>Authenticate app</h1>
      </Match>
      <Match when={fetchStatus() === FetchStatus.Fetching}>
        <h1>Authenticating...</h1>
      </Match>
      <Match when={fetchStatus() === FetchStatus.Success}>
        <h1>Authenticated</h1>
        <p>token: {token().toString()}</p>
        <a
          style={{
            "margin-top": "1rem",
          }}
          href="/"
        >
          Go back
        </a>
      </Match>
    </Switch>
  );
};

const App: Component = () => {
  return (
    <Routes>
      <Route path="/" component={Comp} />
      <Route path="/login" component={Auth} />
    </Routes>
  );
};

export default App;
