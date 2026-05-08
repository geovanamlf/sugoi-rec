import { useState } from "react"
import { useNavigate, Link } from "react-router-dom"
import { useAuth } from "../context/AuthContext"
import api from "../api/client"

export default function Login() {
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState(null)
  const { login } = useAuth()
  const navigate = useNavigate()

  async function handleSubmit(e) {
    e.preventDefault()
    setError(null)
    try {
      const params = new URLSearchParams()
      params.append("username", email)
      params.append("password", password)
      const res = await api.post("/auth/login", params)
      login(res.data.access_token)
      navigate("/dashboard")
    } catch {
      setError("Email ou senha inválidos.")
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <div className="pixel-box w-full max-w-sm">
        <h1 className="pixel-title text-center mb-1">SUGOI REC</h1>
        <p className="text-center text-sm mb-8" style={{ color: "#c9a87c" }}>entre na sua conta</p>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <input className="pixel-input" type="email" placeholder="email" value={email}
            onChange={(e) => setEmail(e.target.value)} />
          <input className="pixel-input" type="password" placeholder="senha" value={password}
            onChange={(e) => setPassword(e.target.value)} />
          {error && <p className="text-sm text-center" style={{ color: "#e07070" }}>✗ {error}</p>}
          <button className="pixel-btn w-full mt-2" type="submit">» ENTRAR</button>
        </form>

        <p className="text-center text-sm mt-6" style={{ color: "#a8a8c0" }}>
          sem conta?{" "}
          <Link to="/register" style={{ color: "#a8c5a0" }}>registrar</Link>
        </p>
      </div>
    </div>
  )
}