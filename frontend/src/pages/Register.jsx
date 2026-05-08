import { useState } from "react"
import { useNavigate, Link } from "react-router-dom"
import api from "../api/client"

export default function Register() {
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState(null)
  const navigate = useNavigate()

  async function handleSubmit(e) {
    e.preventDefault()
    setError(null)
    try {
      await api.post("/auth/register", { email, password })
      navigate("/login")
    } catch {
      setError("Erro ao registrar. Tente outro email.")
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <div className="pixel-box w-full" style={{ maxWidth: "400px" }}>
        <h1 className="pixel-title text-center mb-1">SUGOI REC</h1>
        <p className="text-center text-sm mb-8" style={{ color: "#c9a87c" }}>crie sua conta</p>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <input className="pixel-input" type="email" placeholder="email" value={email}
            onChange={(e) => setEmail(e.target.value)} />
          <input className="pixel-input" type="password" placeholder="senha" value={password}
            onChange={(e) => setPassword(e.target.value)} />
          {error && <p className="text-sm text-center" style={{ color: "#e07070" }}>✗ {error}</p>}
          <button className="pixel-btn w-full mt-2" type="submit">» REGISTRAR</button>
        </form>

        <p className="text-center text-sm mt-6" style={{ color: "#a8a8c0" }}>
          já tem conta?{" "}
          <Link to="/login" style={{ color: "#a8c5a0" }}>login</Link>
        </p>
      </div>
    </div>
  )
}