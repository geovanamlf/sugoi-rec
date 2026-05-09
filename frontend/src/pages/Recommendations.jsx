import { useEffect, useState } from "react"
import { useNavigate } from "react-router-dom"
import api from "../api/client"

export default function Recommendations() {
  const [recs, setRecs] = useState([])
  const [loading, setLoading] = useState(true)
  const [adding, setAdding] = useState(null)
  const navigate = useNavigate()

  useEffect(() => {
    api.get("/recommendations/")
      .then((res) => setRecs(res.data))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="min-h-screen">
      <div className="page-container">

        <div style={{ display: "flex", alignItems: "center", gap: "1rem", marginBottom: "2rem" }}>
          <button className="pixel-btn" onClick={() => navigate("/dashboard")}>« VOLTAR</button>
          <h1 className="pixel-title" style={{ fontSize: "36px" }}>RECOMENDAÇÕES</h1>
        </div>

        {loading && (
          <div className="pixel-box" style={{ maxWidth: "300px" }}>
            <p className="font-pixel" style={{ fontSize: "20px", color: "#c9a87c" }}>⌛ carregando...</p>
          </div>
        )}

        {!loading && recs.length === 0 && (
          <div className="pixel-box" style={{ maxWidth: "400px" }}>
            <p style={{ fontSize: "14px", color: "#a8a8c0" }}>
              nenhuma recomendação ainda.<br /><br />
              adicione animes à sua lista primeiro!
            </p>
          </div>
        )}

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))", gap: "1rem" }}>
          {recs.map((anime) => (
            <div
              key={anime.anilist_id}
              className="pixel-box"
              style={{ padding: "0.75rem", display: "flex", flexDirection: "column", gap: "0.5rem", cursor: "pointer" }}
              onClick={() => navigate(`/anime/${anime.anilist_id}`)}
            >
              {anime.cover_image_url && (
                <img
                  src={anime.cover_image_url}
                  alt={anime.title_romaji}
                  style={{ width: "100%", height: "180px", objectFit: "cover", border: "3px solid #e8d5b7" }}
                />
              )}
              <h2 className="font-pixel" style={{ fontSize: "14px", color: "#a8c5a0", lineHeight: "1.4" }}>
                {anime.title_english || anime.title_romaji}
              </h2>
              <p style={{ fontSize: "12px", color: "#c9a87c" }}>
                {anime.episodes ? `${anime.episodes} eps` : "eps: —"}
              </p>
              <p style={{ fontSize: "12px", color: "#a8a8c0" }}>
                {anime.genres?.join(" · ")}
              </p>
              <button
                className="pixel-btn"
                style={{ fontSize: "13px", padding: "6px", marginTop: "auto" }}
                onClick={(e) => { e.stopPropagation(); setAdding(anime) }}
              >
                + ADICIONAR
              </button>
            </div>
          ))}
        </div>

      </div>

      {adding && (
        <AddModal
          anime={adding}
          onClose={() => setAdding(null)}
        />
      )}
    </div>
  )
}

function AddModal({ anime, onClose }) {
  const [status, setStatus] = useState("planned")
  const [rating, setRating] = useState("")
  const [favorite, setFavorite] = useState(false)
  const [success, setSuccess] = useState(false)
  const [error, setError] = useState(null)

  async function handleAdd() {
    setError(null)
    try {
      const res = await api.get(`/anime/id/${anime.anilist_id}`)
      const animeData = res.data

      await api.post("/list/", {
        anime_id: animeData.id,
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
    <div style={{
      position: "fixed", inset: 0, background: "rgba(0,0,0,0.8)",
      display: "flex", alignItems: "center", justifyContent: "center",
      zIndex: 50, padding: "1rem"
    }}>
      <div className="pixel-box" style={{ width: "100%", maxWidth: "400px" }}>

        <div style={{ display: "flex", gap: "1rem", marginBottom: "1.5rem", flexWrap: "wrap" }}>
          {anime.cover_image_url && (
            <img
              src={anime.cover_image_url}
              alt={anime.title_romaji}
              style={{ width: "80px", border: "3px solid #e8d5b7", alignSelf: "flex-start" }}
            />
          )}
          <div style={{ flex: 1 }}>
            <h2 className="font-pixel" style={{ fontSize: "14px", color: "#a8c5a0", lineHeight: "1.6", marginBottom: "0.5rem" }}>
              {anime.title_english || anime.title_romaji}
            </h2>
            <p style={{ fontSize: "12px", color: "#c9a87c" }}>
              {anime.episodes ? `${anime.episodes} eps` : "eps: —"}
            </p>
            <p style={{ fontSize: "12px", color: "#a8a8c0" }}>
              {anime.genres?.join(" · ")}
            </p>
          </div>
        </div>

        {success ? (
          <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
            <p style={{ fontSize: "14px", color: "#a8c5a0" }}>✓ adicionado à lista!</p>
            <button className="pixel-btn" onClick={onClose}>✕ fechar</button>
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
            <div>
              <p style={{ fontSize: "13px", color: "#c9a87c", marginBottom: "0.5rem" }}>status:</p>
              <select className="pixel-input" value={status} onChange={(e) => setStatus(e.target.value)}>
                <option value="planned">planejado</option>
                <option value="watching">assistindo</option>
                <option value="completed">completado</option>
                <option value="dropped">dropado</option>
              </select>
            </div>

            <div>
              <p style={{ fontSize: "13px", color: "#c9a87c", marginBottom: "0.5rem" }}>nota (1-10):</p>
              <input
                className="pixel-input"
                type="number"
                placeholder="opcional"
                value={rating}
                onChange={(e) => setRating(e.target.value)}
                min={1} max={10}
              />
            </div>

            <label style={{ display: "flex", alignItems: "center", gap: "0.75rem", fontSize: "13px", cursor: "pointer" }}>
              <input type="checkbox" checked={favorite} onChange={(e) => setFavorite(e.target.checked)} />
              <span style={{ color: "#e8d5b7" }}>♥ favorito</span>
            </label>

            {error && <p style={{ fontSize: "13px", color: "#e07070" }}>✗ {error}</p>}

            <div style={{ display: "flex", gap: "0.75rem" }}>
              <button className="pixel-btn" style={{ flex: 1 }} onClick={handleAdd}>
                + ADICIONAR
              </button>
              <button className="pixel-btn pixel-btn-danger" onClick={onClose}>
                ✕
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}