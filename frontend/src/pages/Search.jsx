import { useState } from "react"
import { useNavigate } from "react-router-dom"
import api from "../api/client"

export default function Search() {
  const [query, setQuery] = useState("")
  const [result, setResult] = useState(null)
  const [error, setError] = useState(null)
  const [status, setStatus] = useState("planned")
  const [rating, setRating] = useState("")
  const [favorite, setFavorite] = useState(false)
  const [success, setSuccess] = useState(false)
  const navigate = useNavigate()

  async function handleSearch(e) {
    e.preventDefault()
    setError(null)
    setResult(null)
    setSuccess(false)
    try {
      const res = await api.get(`/anime/search?q=${query}`)
      setResult(res.data)
    } catch {
      setError("anime não encontrado.")
    }
  }

  async function handleAdd() {
    try {
      await api.post("/list/", {
        anime_id: result.id,
        status,
        rating: rating ? parseInt(rating) : null,
        is_favorite: favorite,
      })
      setSuccess(true)
    } catch {
      setError("erro ao adicionar. talvez já esteja na lista.")
    }
  }

  return (
    <div className="min-h-screen">
      <div className="page-container" style={{ maxWidth: "600px" }}>

        <div style={{ display: "flex", alignItems: "center", gap: "1rem", marginBottom: "2rem" }}>
          <button className="pixel-btn" onClick={() => navigate("/dashboard")}>« VOLTAR</button>
          <h1 className="pixel-title" style={{ fontSize: "36px" }}>BUSCAR</h1>
        </div>

        <div className="pixel-box" style={{ marginBottom: "1.5rem" }}>
          <form onSubmit={handleSearch} style={{ display: "flex", gap: "0.75rem", flexWrap: "wrap" }}>
            <input className="pixel-input" placeholder="nome do anime..." value={query}
              onChange={(e) => setQuery(e.target.value)} style={{ flex: 1, minWidth: "200px" }} />
            <button className="pixel-btn" type="submit">» BUSCAR</button>
          </form>
          {error && <p style={{ fontSize: "14px", color: "#e07070", marginTop: "0.75rem" }}>✗ {error}</p>}
        </div>

        {result && (
          <div className="pixel-box">
            <div style={{ display: "flex", gap: "1rem", marginBottom: "1.5rem", flexWrap: "wrap" }}>
              {result.cover_image_url && (
                <img src={result.cover_image_url} alt={result.title_romaji}
                  style={{ width: "100px", border: "3px solid #e8d5b7", alignSelf: "flex-start" }} />
              )}
              <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", flex: 1 }}>
                <h2 className="font-pixel" style={{ fontSize: "22px", color: "#a8c5a0", lineHeight: "1.4" }}>
                  {result.title_english || result.title_romaji}
                </h2>
                <p style={{ fontSize: "14px", color: "#c9a87c" }}>
                  {result.episode_count ? `${result.episode_count} episódios` : "episódios: —"}
                </p>
                <p style={{ fontSize: "13px", color: "#a8a8c0" }}>
                  {result.genres?.replace(/,/g, " · ")}
                </p>
              </div>
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
              <div>
                <p style={{ fontSize: "14px", color: "#c9a87c", marginBottom: "0.5rem", fontWeight: 600 }}>status:</p>
                <select className="pixel-input" value={status} onChange={(e) => setStatus(e.target.value)}>
                  <option value="planned">planejado</option>
                  <option value="watching">assistindo</option>
                  <option value="completed">completado</option>
                  <option value="dropped">dropado</option>
                </select>
              </div>

              <div>
                <p style={{ fontSize: "14px", color: "#c9a87c", marginBottom: "0.5rem", fontWeight: 600 }}>nota (1-10):</p>
                <input className="pixel-input" type="number" placeholder="opcional"
                  value={rating} onChange={(e) => setRating(e.target.value)} min={1} max={10} />
              </div>

              <label style={{ display: "flex", alignItems: "center", gap: "0.75rem", fontSize: "14px", cursor: "pointer" }}>
                <input type="checkbox" checked={favorite} onChange={(e) => setFavorite(e.target.checked)} />
                <span style={{ color: "#e8d5b7" }}>♥ favorito</span>
              </label>

              {success ? (
                <p style={{ fontSize: "14px", color: "#a8c5a0" }}>
                  ✓ adicionado!{" "}
                  <span style={{ cursor: "pointer", textDecoration: "underline" }} onClick={() => navigate("/dashboard")}>
                    ver lista
                  </span>
                </p>
              ) : (
                <button className="pixel-btn" onClick={handleAdd}>+ ADICIONAR À LISTA</button>
              )}
            </div>
          </div>
        )}

      </div>
    </div>
  )
}