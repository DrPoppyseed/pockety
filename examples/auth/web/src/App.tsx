import axios from 'axios';
import { type Component } from 'solid-js';

const App: Component = () => {

  const onLoginClick = async () => {
    const result = await axios.get('http://localhost:3000/login');
    console.log({ result })
  }

  return (
    <>
      <h1>Auth Example</h1>
      <button onClick={() => onLoginClick()}>Login with Pocket</button>
    </>
  );
};

export default App;
