const WIDTH = 500;
const HEIGHT = 500;

let session = null;
let state = null;
let animationFrameReq = null;
let pollGameInterval = null;
let previousFrameTime = null;
let controlsPressed = {
  x: 0,
  y: 0,
};

function main() {
  setupRegistration();
}

function normalize(vec) {
  const len = Math.sqrt(vec.x * vec.x + vec.y * vec.y);
  if (len === 0) {
    return vec;
  }
  return { x: vec.x / len, y: vec.y / len };
}

function drawCircle(ctx, position, color) {
  ctx.beginPath();
  ctx.arc(position.x, position.y, 20, 0, 2 * Math.PI);
  ctx.fillStyle = color;
  ctx.fill();
  ctx.closePath();
}

function renderGame(timestamp) {
  if (state === null) {
    animationFrameReq = requestAnimationFrame(renderGame);
    return;
  }

  const dt = (previousFrameTime ? timestamp - previousFrameTime : 0) / 1000;
  previousFrameTime = timestamp;
  const control = normalize(controlsPressed);
  session.player.position.x += dt * control.x * 50;
  session.player.position.y += dt * control.y * 50;

  const ctx = document.querySelector("#gameCanvas").getContext("2d");
  ctx.globalCompositeOperation = "destination-over";

  ctx.clearRect(0, 0, WIDTH, HEIGHT);

  drawCircle(ctx, session.player.position, "red");

  for (const player of state) {
    drawCircle(ctx, player.position, "green");
  }

  animationFrameReq = requestAnimationFrame(renderGame);
}

function keyDownHandler(e) {
  if (e.key === "Left" || e.key === "ArrowLeft") {
    controlsPressed.x = -1;
  } else if (e.key === "Right" || e.key === "ArrowRight") {
    controlsPressed.x = 1;
  } else if (e.key === "Up" || e.key === "ArrowUp") {
    controlsPressed.y = -1;
  } else if (e.key === "Down" || e.key === "ArrowDown") {
    controlsPressed.y = 1;
  }
}

function keyUpHandler(e) {
  if (e.key === "Left" || e.key === "ArrowLeft") {
    controlsPressed.x = 0;
  } else if (e.key === "Right" || e.key === "ArrowRight") {
    controlsPressed.x = 0;
  } else if (e.key === "Up" || e.key === "ArrowUp") {
    controlsPressed.y = 0;
  } else if (e.key === "Down" || e.key === "ArrowDown") {
    controlsPressed.y = 0;
  }
}

async function getData(url) {
  const response = await fetch(url);
  return response.json();
}

async function postData(url, data) {
  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });
  return response.json();
}

async function updateData(url, data) {
  data.secret = session.secret;
  const response = await fetch(url, {
    method: "PATCH",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });
  return response.json();
}

function pollGame() {
  updateData("players/" + session.player.name, { position: session.player.position }).then((resp) => {
    if (Array.isArray(resp)) {
      state = resp.filter((p) => p.name !== session.player.name);
    } else {
      failGame(resp.error ? resp.error : "Network error");
    }
  }).catch(() => failGame("Network error"));
}

function failGame(message) {
  if (animationFrameReq) {
    cancelAnimationFrame(animationFrameReq);
  }
  if (pollGameInterval) {
    clearInterval(pollGameInterval);
  }
  document.querySelector("#gameCanvas").remove();
  const errorMessage = document.createElement("p");
  errorMessage.setAttribute("class", "errorMessage");
  errorMessage.textContent = message;
  document.querySelector("#root").appendChild(errorMessage);
}

function startGame(registrationResponse) {
  session = registrationResponse;

  // Set up canvas
  const root = document.querySelector("#root");
  const canvas = document.createElement("canvas");
  canvas.setAttribute("id", "gameCanvas");
  canvas.setAttribute("width", WIDTH.toString());
  canvas.setAttribute("height", HEIGHT.toString());
  root.appendChild(canvas);

  // Set up state polling
  pollGame();
  pollGameInterval = setInterval(pollGame, 100);

  // Set up rendering
  animationFrameReq = requestAnimationFrame(renderGame);

  // Set up controls
  addEventListener("keydown", keyDownHandler, false);
  addEventListener("keyup", keyUpHandler, false);
}

function handleStartButton(e) {
  e.preventDefault();

  let name = document.querySelector("#nameField");
  const createPlayer = { name: name.value };
  console.log(createPlayer);

  let reg = document.querySelector("#registration");
  postData("players", createPlayer).then((resp) => {
    if (resp && resp.player && resp.secret) {
      reg.remove();
      startGame(resp);
    } else {
      name.value = "";
      let errorMessage = document.querySelector(".errorMessage");
      if (errorMessage === null) {
        errorMessage = document.createElement("p");
        errorMessage.setAttribute("class", "errorMessage");
        reg.appendChild(errorMessage);
      }
      errorMessage.textContent = resp.error ? resp.error : "Request failed";
      if (errorMessage._my_timeout) {
        clearTimeout(errorMessage._my_timeout);
      }
      errorMessage._my_timeout = setTimeout(() => { errorMessage.remove() }, 3000);
    }
  });
}

function setupRegistration() {
  const reg = document.createElement("div");
  reg.setAttribute("id", "registration");

  const title = document.createElement("h1");
  title.setAttribute("class", "title");
  title.textContent = "Wappu Game Jam Multiplayer something something 2024";
  reg.appendChild(title);

  const form = document.createElement("form");
  const labelName = document.createElement("label");
  labelName.textContent = "Display name:";
  form.appendChild(labelName);
  const inputName = document.createElement("input");
  inputName.setAttribute("type", "text");
  inputName.setAttribute("id", "nameField");
  form.appendChild(inputName);
  const startButton = document.createElement("input");
  startButton.setAttribute("type", "submit");
  startButton.setAttribute("value", "Start");
  form.appendChild(startButton);
  form.addEventListener("submit", handleStartButton);
  reg.appendChild(form);

  const root = document.querySelector("#root");
  root.appendChild(reg);
}

main()
