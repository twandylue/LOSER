async function search(query) {
  const results = document.getElementById("results");
  results.innerHTML = "";
  const response = await fetch("/api/search", {
    method: "POST",
    headers: { "Content-Type": "text/plain" },
    body: query,
  })
  const json = await response.json();
  for ([path, rank] of json) {
    let item = document.createElement("span");
    item.appendChild(document.createTextNode(path));
    item.appendChild(document.createTextNode(" | "));
    item.appendChild(document.createTextNode("rank: " + rank));
    item.appendChild(document.createElement("br"));
    results.appendChild(item)
  }
}

let query = document.getElementById("query");
query.addEventListener("keypress", async (e) => {
  if (e.key == "Enter") {
    await search(query.value)
  }
})
