import { useState } from "react"
import { useNavigate } from "react-router-dom"
import api from "../api/client"

export default function Search() {
  const [query, setQuery] = useState("")
  const [result, setResult] = useState(null)
  const [error, setError] = useState(null)
  const [status, setStatus] = useState("planned")
  const [rating, setRating] = useState("")
  const navigate = useNavigate()

  async function handleSearch(e) {
    e.preventDefault()
    setError(null)
    setResult(null)
    try {
      const res = await api.get(`/anime/search?q=${query}`)
      setResult(res.data)
    } catch {
      setError("Anime não encontrado.")
    }
  }

  async function handleAdd() {
    try {
      await api.post("/list/", {
        anime_id: result.id,
        status,
        rating: rating ? parseInt(rating) : null,
        is_favorite: false,
      })
      navigate("/dashboard")
    } catch {
      setError("Erro ao adicionar. Talvez já esteja na lista.")
    }
  }

  return (
    <div>
      <h1>Buscar Anime</h1>
      <button onClick={() => navigate("/dashboard")}>Voltar</button>
      <form onSubmit={handleSearch}>
        <input
          placeholder="Nome do anime"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <button type="submit">Buscar</button>
      </form>

      {error && <p>{error}</p>}

      {result && (
        <div>
          <h2>{result.title_english || result.title_romaji}</h2>
          <img src={result.cover_image_url} alt={result.title_romaji} width={100} />
          <p>{result.genres}</p>
          <p>{result.episode_count} episódios</p>

          <select value={status} onChange={(e) => setStatus(e.target.value)}>
            <option value="planned">Planejado</option>
            <option value="watching">Assistindo</option>
            <option value="completed">Completado</option>
            <option value="dropped">Dropado</option>
          </select>

          <input
            type="number"
            placeholder="Nota (1-10)"
            value={rating}
            onChange={(e) => setRating(e.target.value)}
            min={1}
            max={10}
          />

          <button onClick={handleAdd}>Adicionar à lista</button>
        </div>
      )}
    </div>
  )
}
