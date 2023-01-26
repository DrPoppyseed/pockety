import { Route, Routes, useLocation, useNavigate } from "@solidjs/router";
import axios from "axios";
import {
  createEffect,
  createSignal,
  Match,
  onMount,
  Show,
  Switch,
  type Component,
} from "solid-js";

const FetchStatus = {
  Idle: "idle",
  Fetching: "fetching",
  Success: "success",
  Error: "error",
} as const;
type FetchStatusType = typeof FetchStatus[keyof typeof FetchStatus];

const Comp: Component = () => {
  const [data, setData] = createSignal<String | null>(null);

  const onLoginClick = async () => {
    const result = await axios.post("http://localhost:8080/auth/pocket");

    if (result.status === 200) {
      setData(result.data);
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
      <Show when={data()}>
        <a
          style={{
            "margin-bottom": "1rem",
          }}
          href={data().toString()}
        >
          Grant permissions for Pocket
        </a>
      </Show>
    </div>
  );
};

const Auth: Component = () => {
  const navigate = useNavigate();
  const [fetchStatus, setFetchStatus] = createSignal<FetchStatusType>(
    FetchStatus.Idle
  );

  onMount(async () => {
    setFetchStatus(FetchStatus.Fetching);

    const result = await axios.post("http://localhost:8080/auth/authorize");

    if (result.status === 200) {
      setFetchStatus(FetchStatus.Success);
    } else {
      setFetchStatus(FetchStatus.Error);
    }
  });

  createEffect(() => {
    if (fetchStatus() === FetchStatus.Success) {
      console.log("success");
      navigate("/");
    } else if (fetchStatus() === FetchStatus.Error) {
      console.log("error");
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
