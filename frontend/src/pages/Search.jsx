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
    <div className="min-h-screen p-4 md:p-8">
      <div className="max-w-2xl mx-auto">

        <div className="flex items-center gap-4 mb-8">
          <button className="pixel-btn" onClick={() => navigate("/dashboard")}>« VOLTAR</button>
          <h1 className="pixel-title text-3xl">BUSCAR</h1>
        </div>

        <div className="pixel-box mb-6">
          <form onSubmit={handleSearch} className="flex gap-3 flex-col sm:flex-row">
            <input className="pixel-input" placeholder="nome do anime..." value={query}
              onChange={(e) => setQuery(e.target.value)} />
            <button className="pixel-btn whitespace-nowrap" type="submit">» BUSCAR</button>
          </form>
          {error && <p className="text-sm mt-3" style={{ color: "#e07070" }}>✗ {error}</p>}
        </div>

        {result && (
          <div className="pixel-box">
            <div className="flex gap-4 mb-6 flex-col sm:flex-row">
              {result.cover_image_url && (
                <img src={result.cover_image_url} alt={result.title_romaji}
                  className="w-28 h-auto self-start"
                  style={{ border: "3px solid #e8d5b7" }} />
              )}
              <div className="flex flex-col gap-2">
                <h2 className="font-pixel text-2xl leading-8" style={{ color: "#a8c5a0" }}>
                  {result.title_english || result.title_romaji}
                </h2>
                <p className="text-sm" style={{ color: "#c9a87c" }}>
                  {result.episode_count ? `${result.episode_count} episódios` : "episódios: —"}
                </p>
                <p className="text-sm" style={{ color: "#a8a8c0" }}>
                  {result.genres?.replace(/,/g, " · ")}
                </p>
              </div>
            </div>

            <div className="flex flex-col gap-4">
              <div>
                <p className="text-sm mb-2 font-semibold" style={{ color: "#c9a87c" }}>status:</p>
                <select className="pixel-input" value={status} onChange={(e) => setStatus(e.target.value)}>
                  <option value="planned">planejado</option>
                  <option value="watching">assistindo</option>
                  <option value="completed">completado</option>
                  <option value="dropped">dropado</option>
                </select>
              </div>

              <div>
                <p className="text-sm mb-2 font-semibold" style={{ color: "#c9a87c" }}>nota (1-10):</p>
                <input className="pixel-input" type="number" placeholder="opcional"
                  value={rating} onChange={(e) => setRating(e.target.value)} min={1} max={10} />
              </div>

              <label className="flex items-center gap-3 text-sm cursor-pointer">
                <input type="checkbox" checked={favorite} onChange={(e) => setFavorite(e.target.checked)} />
                <span style={{ color: "#e8d5b7" }}>♥ favorito</span>
              </label>

              {success ? (
                <p className="text-sm" style={{ color: "#a8c5a0" }}>
                  ✓ adicionado!{" "}
                  <span className="cursor-pointer underline" onClick={() => navigate("/dashboard")}>
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