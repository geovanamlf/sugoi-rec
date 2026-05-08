import { useEffect, useState } from "react"
import { useNavigate } from "react-router-dom"
import api from "../api/client"

export default function Recommendations() {
  const [recs, setRecs] = useState([])
  const [loading, setLoading] = useState(true)
  const navigate = useNavigate()

  useEffect(() => {
    api.get("/recommendations/")
      .then((res) => setRecs(res.data))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="min-h-screen p-4 md:p-8">
      <div className="max-w-6xl mx-auto">

        <div className="flex items-center gap-4 mb-8">
          <button className="pixel-btn" onClick={() => navigate("/dashboard")}>« VOLTAR</button>
          <h1 className="pixel-title text-3xl">RECOMENDAÇÕES</h1>
        </div>

        {loading && (
          <div className="pixel-box max-w-xs">
            <p className="font-pixel text-xl" style={{ color: "#c9a87c" }}>⌛ carregando...</p>
          </div>
        )}

        {!loading && recs.length === 0 && (
          <div className="pixel-box max-w-sm">
            <p className="text-sm" style={{ color: "#a8a8c0" }}>
              nenhuma recomendação ainda.<br /><br />
              adicione animes à sua lista primeiro!
            </p>
          </div>
        )}

        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          {recs.map((anime) => (
            <div key={anime.anilist_id} className="pixel-box p-3 flex flex-col gap-2">
              {anime.cover_image_url && (
                <img src={anime.cover_image_url} alt={anime.title_romaji}
                  className="w-full h-44 object-cover"
                  style={{ border: "3px solid #e8d5b7" }} />
              )}
              <h2 className="font-pixel text-lg leading-6" style={{ color: "#a8c5a0" }}>
                {anime.title_english || anime.title_romaji}
              </h2>
              <p className="text-xs" style={{ color: "#c9a87c" }}>
                {anime.episodes ? `${anime.episodes} eps` : "eps: —"}
              </p>
              <p className="text-xs" style={{ color: "#a8a8c0" }}>
                {anime.genres?.join(" · ")}
              </p>
            </div>
          ))}
        </div>

      </div>
    </div>
  )
}