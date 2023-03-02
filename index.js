console.log("Hello World!")

async function search(prompt) {
  // TODO:
  // const results = document.getElementById("results");
  // results.innerHTML = "";
  const response = await fetch("/api/search", {
    method: "POST",
    headers: { "Content-Type": "text/plain" },
    body: prompt,
  })
  const json = await response.json();

  console.log(json);
}

search("github")
