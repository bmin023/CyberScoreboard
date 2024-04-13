import { useState } from "react";
import { useLogin } from "../Hooks/CtrlHooks";
import { Link, useNavigate } from 'react-router-dom'

const LoginPage = () => {
  const { login, isLoginError } = useLogin()
  const [username, setUsername] = useState(new URLSearchParams(window.location.search).get('user') ?? "")
  const [password, setPassword] = useState("")
  const navigate = useNavigate()

  const processLogin = () => {
    const data = {
      username,
      password
    } as { [name: string]: string }
    const redir = new URLSearchParams(window.location.search).get('redirect')
    if (redir) {
      data["redirect"] = redir
    }
    login(data, {
      onSuccess: () => {
        if (redir) {
          navigate(redir)
        } else {
          navigate("/")
        }
      }
    })
  }
  return (
    <div className="h-screen">
      <Link className="underline m-1" to="/">Back Home</Link>
      <div className="p-2 h-full flex flex-col justify-center">
        <h1 className="text-center text-5xl">Login</h1>
        <div className="my-5 flex justify-center items-center">
          <form onSubmit={e => { e.preventDefault(); processLogin() }}
            className="flex flex-col gap-3"
          >
            <input className="px-2 py-1 text-center text-xl shadow dark:bg-zinc-800" value={username} onChange={e => setUsername(e.target.value)} />
            <input className="px-2 py-1 text-center text-xl shadow dark:bg-zinc-800" value={password} onChange={e => setPassword(e.target.value)} type="password" />
            <input className="py-1 text-xl border border-slate-200 shadow rounded hover:shadow-lg active:shadow-sm" value={"Login"} type="submit" />
          </form>
        </div>
        {isLoginError && <p className="text-center text-red-500">There was an error logging in. Are these the right credentials?</p>}
      </div>
    </div>
  );
};

export default LoginPage
