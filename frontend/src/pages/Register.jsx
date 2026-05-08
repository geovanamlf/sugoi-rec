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
    } catch (err) {
      setError("Erro ao registrar. Tente outro email.")
    }
  }

  return (
    <div>
      <h1>Registrar</h1>
      <form onSubmit={handleSubmit}>
        <input
          type="email"
          placeholder="Email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
        <input
          type="password"
          placeholder="Senha"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        {error && <p>{error}</p>}
        <button type="submit">Registrar</button>
      </form>
      <p>Já tem conta? <Link to="/login">Login</Link></p>
    </div>
  )
}
