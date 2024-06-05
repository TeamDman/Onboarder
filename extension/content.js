let onboarder_id = "not setup yet"
let tag = `[Onboarder-${onboarder_id}]`;
const textAreaId = "custom_notes_area";

let serverUrl = "https://localhost:5876/";
chrome.storage.local.get("serverUrl", function (data) {
    serverUrl = data.serverUrl || defaultServerUrl;
    console.log(`${tag} serverUrl is ${serverUrl}`);
});
chrome.storage.onChanged.addListener(function (changes, namespace) {
    for (let key in changes) {
        if (key === "serverUrl") {
            serverUrl = changes[key].newValue.replace(/\/$/, "");
            console.log(`${tag} serverUrl is ${serverUrl}`);
        } else if (key === "refresh") {
            console.log(`${tag} refresh received, setting up`);
            setup();
        }
    }
});

function removeElementById(id) {
    let element = document.getElementById(id);
    if (element) {
        element.parentNode.removeChild(element);
    }
}

function getLocalDate() {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, "0"); // Months are zero-based
    const day = String(now.getDate()).padStart(2, "0");

    return `${year}-${month}-${day}`;
}

function getNoteId() {
    const title = document.querySelector("#title.ytd-watch-metadata");
    const videoId = document.querySelector("ytd-watch-metadata").getAttribute("video-id");
    const date = getLocalDate();
    const id = `[${date}] [youtube] [${videoId}] ${title.innerText}`;
    return id;
}

function getCurrentNoteContent() {
    return document.getElementById("custom_notes_area").value;
}

// Function to add the text area and attach an event listener to it
function addTextArea(videoArea, initialContent) {
    console.log(`${tag} adding content to page`);

    // Remove the existing text area if it exists
    removeElementById(textAreaId);

    // Create the text area element
    let textArea = document.createElement("textarea");
    textArea.value = initialContent;

    // Assign a unique ID to the text area
    textArea.id = textAreaId;

    // Style the text area
    textArea.style.width = "calc(100% - 35px)";
    textArea.style.minHeight = "200px";
    textArea.style.padding = "10px";
    textArea.style.marginTop = "20px";
    textArea.style.marginLeft = "5px";
    textArea.style.borderRadius = "8px";
    textArea.style.backgroundColor = "#1f1f1f";
    textArea.style.color = "#ffffff";
    textArea.placeholder = "notes";

    // Add a change event listener to send the content to the Rust endpoint
    textArea.addEventListener("input", function () {
        save(this.value);
    });

    // Insert the text area after the video player element
    videoArea.insertAdjacentElement("afterend", textArea);
}

async function appendContent(content) {
    const existing = getCurrentNoteContent();
    if (!existing.endsWith("\n")) content = "\n" + content;
    if (!content.endsWith("\n")) content += "\n";
    const next = existing + content;
    console.log(`${tag} appending content`, {existing, next});
    await save(next);
    const note = document.getElementById("custom_notes_area");
    note.value = next;
}

function getVideoProgress() {
    const video = document.getElementsByClassName("html5-main-video")[0];
    // in seconds
    const current = video.currentTime;
    const duration = video.duration;
    
    // convert to hh:mm:ss / hh:mm:ss (%%%) format
    const currentFormatted = new Date(current * 1000).toISOString().substr(11, 8);
    const durationFormatted = new Date(duration * 1000).toISOString().substr(11, 8);
    const percentage = (current / duration * 100).toFixed(2);
    return `${currentFormatted} / ${durationFormatted} (${percentage}%)`;
}

function addChips(videoArea) {
    console.log(`${tag} building action chips`);

    removeElementById("chip_container");
    const chipContainer = document.createElement("div");
    chipContainer.id = "chip_container";
    chipContainer.style.display = "flex";
    chipContainer.style.flexWrap = "wrap";

    const actions = [
        {
            text: "Timestamp",
            description: "Insert the current video timestamp",
            action: async function () {
                console.log(`${tag} Inserting timestamp`);
                await appendContent(
                    `\nvideo current time ${getVideoProgress()} at ${new Date().toString()}`
                );
            },
        },
        {
            text: "Download",
            description: "Save the video to disk",
            action: async function () {
                await downloadVideo();
            },
        },
        {
            text: "Download (🎶 🚫📷)",
            description: "Download without video",
            action: async function() {
                await downloadAudio();
            }
        },
        {
            text: "📁 Videos",
            description: "Open videos folder",
            action: async function() {
                await openVideosFolder();
            }
        },
        {
            text: "📁 Notes",
            description: "Open notes folder",
            action: async function() {
                await openNotesFolder();
            }
        },
        {
            text: "📝 Transcript",
            description: "Open transcript website",
            action: async function() {
                openTranscriptInNewTab();
            }
        }
    ];
    actions.forEach((action) => {
        const chip = document.createElement("button");
        chip.innerText = action.text;
        chip.title = action.description;
        chip.style.margin = "5px";
        chip.style.padding = "10px";
        chip.style.borderRadius = "12px";
        chip.style.cursor = "pointer";
        chip.style.backgroundColor = "#1f1f1f";
        chip.style.color = "#ffffff";

        chip.addEventListener("click", action.action);

        chipContainer.appendChild(chip);
    });

    videoArea.insertAdjacentElement("afterend", chipContainer);
}

async function onPause() {
    if (window.onboarder_id != onboarder_id) return;
    console.log(`${tag} video paused`, {time: getVideoProgress(), noteId: getNoteId()});
    await appendContent(
        `${new Date().toString()} --- ${getVideoProgress()} --- paused`
    );
}

async function onPlaying() {
    if (window.onboarder_id != onboarder_id) return;
    console.log(`${tag} video playing`, {time: getVideoProgress(), noteId: getNoteId()});
    await appendContent(
        `${new Date().toString()} --- ${getVideoProgress()} --- playing`
    );
}

async function onStart() {
    if (window.onboarder_id != onboarder_id) return;
    console.log(`${tag} video started`, {time: getVideoProgress(), noteId: getNoteId()});
    await appendContent(
        `${new Date().toString()} --- ${getVideoProgress()} --- started`
    );
}

async function onStop() {
    if (window.onboarder_id != onboarder_id) return;
    console.log(`${tag} video stopped`, {time: getVideoProgress(), noteId: getNoteId()});
    window.onboarder_id = null;
    await appendContent(
        `${new Date().toString()} --- ${getVideoProgress()} --- stopped`
    );
    const textArea = document.getElementById(textAreaId);
    if (textArea) {
        textArea.readOnly = true;
        textArea.style.color = "gray";
    }
}

function attachPauseAndPlayListeners(videoArea) {
    console.log(`${tag} attaching pause and play listeners`);
    video = videoArea.querySelector("video");

    video.addEventListener("pause", onPause);
    video.addEventListener("playing", onPlaying);
}

async function onLike() {
    if (window.onboarder_id != onboarder_id) return;
    console.log(`${tag} video liked`, {time: getVideoProgress(), noteId: getNoteId()});
    await appendContent(
        `${new Date().toString()} --- ${getVideoProgress()} --- 👍`
    );
}

async function onUnlike() {
    if (window.onboarder_id != onboarder_id) return;
    console.log(`${tag} video unliked`, {time: getVideoProgress(), noteId: getNoteId()});
    await appendContent(
        `${new Date().toString()} --- ${getVideoProgress()} --- ➖👍`
    );
}


async function onDislike() {
    if (window.onboarder_id != onboarder_id) return;
    console.log(`${tag} video disliked`, {time: getVideoProgress(), noteId: getNoteId()});
    await appendContent(
        `${new Date().toString()} --- ${getVideoProgress()} --- 👎`
    );
}

async function onUndislike() {
    if (window.onboarder_id != onboarder_id) return;
    console.log(`${tag} video undisliked`, {time: getVideoProgress(), noteId: getNoteId()});
    await appendContent(
        `${new Date().toString()} --- ${getVideoProgress()} --- ➖👎`
    );
}

function attachLikeListeners() {
    const likeButton = document.querySelector("like-button-view-model");
    likeButton.addEventListener("click", async () => {
        let pressed = likeButton.querySelector("[aria-pressed=true]") != null;
        if (pressed) {
            await onLike();
        } else {
            await onUnlike();
        }
    });

    const dislikeButton = document.querySelector("dislike-button-view-model");
    dislikeButton.addEventListener("click", async () => {
        let pressed = dislikeButton.querySelector("[aria-pressed=true]") != null;
        if (pressed) {
            await onDislike();
        } else {
            await onUndislike();
        }
    });
    console.log(`${tag} attached like listeners`, likeButton, dislikeButton);
}

function save(content) {
    // Build the note ID from the v= slug + the title of the video
    const id = getNoteId();
    console.log(`${tag} saving`, {id, content});

    // Create a POST request to the Rust HTTP server
    return fetch(`${serverUrl}/set_note`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            id,
            content,
        }),
    })
        .then((response) => response.text())
        .then((data) => {
            console.log(`${tag} Success:`, data);
        })
        .catch((error) => {
            console.error(`${tag} Error:`, error);
        });
}

async function downloadVideo() {
    console.log(`${tag} Ensuring video has not already been downloaded before downloading`);
    {
        const videoId = document.querySelector("ytd-watch-metadata").getAttribute("video-id");
        const resp = await fetch(`${serverUrl}/exists?search=${videoId}`);
        // ensure response is 404
        if (resp.status != 404) {
            console.log(`${tag} Video already downloaded, not downloading again`);
            const text = await resp.text();
            alert(`Video already downloaded!\n${text}`);
            return;
        }
    }


    console.log(`${tag} Downloading video`);
    {
        const resp = await fetch(`${serverUrl}/download`, {
            method: "POST",
            headers: {
                "Content-Type": "application/text",
            },
            body: window.location.href.split("&")[0],
        });
        if (resp.status == 200) {
            const fileId = await resp.text();
            const content =
                getCurrentNoteContent() +
                `\n${new Date().toString()} --- Download started for "${fileId}"`;
            await save(content);
            const note = document.getElementById("custom_notes_area");
            note.value = content;
        } else {
            const content =
                getCurrentNoteContent() +
                `\nFailed to download video, status code: ${resp.status}`;
            const note = document.getElementById("custom_notes_area");
            note.value = content;
        }
    }
}


async function downloadAudio() {
    console.log(`${tag} Ensuring audio has not already been downloaded before downloading`);
    {
        const videoId = document.querySelector("ytd-watch-metadata").getAttribute("video-id");
        const resp = await fetch(`${serverUrl}/exists?search=${videoId}`);
        // ensure response is 404
        if (resp.status != 404) {
            console.log(`${tag} audio already downloaded, not downloading again`);
            const text = await resp.text();
            alert(`Video already downloaded!\n${text}`);
            return;
        }
    }


    console.log(`${tag} Downloading audio`);
    {
        const resp = await fetch(`${serverUrl}/download_audio`, {
            method: "POST",
            headers: {
                "Content-Type": "application/text",
            },
            body: window.location.href.split("&")[0],
        });
        if (resp.status == 200) {
            const fileId = await resp.text();
            const content =
                getCurrentNoteContent() +
                `\n${new Date().toString()} --- Download started for "${fileId}"`;
            await save(content);
            const note = document.getElementById("custom_notes_area");
            note.value = content;
        } else {
            const content =
                getCurrentNoteContent() +
                `\nFailed to download audio, status code: ${resp.status}`;
            const note = document.getElementById("custom_notes_area");
            note.value = content;
        }
    }
}

async function openVideosFolder() {
    console.log(`${tag} Opening videos folder`);
    await fetch(`${serverUrl}/open_videos_folder`, {
        method: "POST",
        headers: {
            "Content-Type": "application/text",
        },
        body: window.location.href.split("&")[0],
    });
}
async function openNotesFolder() {
    console.log(`${tag} Opening notes folder`);
    await fetch(`${serverUrl}/open_notes_folder`, {
        method: "POST",
        headers: {
            "Content-Type": "application/text",
        },
        body: window.location.href.split("&")[0],
    });
}

function openTranscriptInNewTab() {
    // Create a URL object from the current window location
    var currentUrl = new URL(window.location.href);
    
    // Get the value of the 'v' query parameter
    var videoId = currentUrl.searchParams.get('v');

    if (videoId) {
        var transcriptUrl = 'https://youtubetranscript.com/?v=' + videoId;
        
        // Open the transcript URL in a new tab
        window.open(transcriptUrl, '_blank');
    } else {
        console.error('Video ID not found in the URL');
    }
}


async function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function setup() {
    onboarder_id = Math.random().toString(36).substring(7);
    tag = `[Onboarder-${onboarder_id}]`;
    window.onboarder_id = onboarder_id;
    console.log(`${tag} setting up at url ${window.location.href}`);
    {
        console.log(`${tag} waiting for server healthcheck to succeed`);
        let attempt = 0;
        while (true) {
            try {
                const resp = await fetch(`${serverUrl}/healthcheck`);
                if (resp.status == 200) {
                    break;
                }
            } catch (ignored) {}
            const backoff = attempt < 10 ? 50 : 1000;
            console.log(
                `${tag} server healthcheck failed, retrying after ${backoff}ms`
            );
            attempt++;
            await sleep(backoff);
        }
        console.log(`${tag} server found`);
    }

    console.log(`${tag} waiting for video player element`);
    let videoArea = document.getElementById("full-bleed-container");
    while (!videoArea) {
        await sleep(1000);
        videoArea = document.getElementById("full-bleed-container");
    }

    console.log(`${tag} removing old listeners`);
    video = videoArea.querySelector("video");
    video.removeEventListener("pause", onPause);
    video.removeEventListener("playing", onPlaying);

    {
        while (true) {
            const videoId = document.querySelector("ytd-watch-metadata").getAttribute("video-id");
            const v = new URL(window.location).searchParams.get("v");
            if (videoId == v) break;
            console.log(`${tag} video ID doesn't match URL (did we just navigate?), waiting 50ms before continuing setup`);
            await sleep(50);
        }
    }

    {
        console.log(`${tag} getting existing note content with note id ${getNoteId()}`);
        let content = "";
        {
            const resp = await fetch(
                `${serverUrl}/get_note?id=${getNoteId()}`
            );
            const data = await resp.json();
            content = data.content;
            console.log(`${tag} received existing content`, {length: content.length, content});
        }
        addTextArea(videoArea, content);
    }

    addChips(videoArea);
    attachPauseAndPlayListeners(videoArea);
    attachLikeListeners();
    {
        const video = videoArea.querySelector("video");
        if (!video.paused) {
            console.log(`${tag} detected autoplay`);
            await onStart();
        }
    }
    console.log(`${tag} setup complete`);
}
//// initial call not needed since yt-navigate-finish event is fired on page load
// setup();

// https://stackoverflow.com/a/34100952/11141271
addEventListener("yt-navigate-start", e => {
    console.log(`${tag} yt-navigate-start event fired, calling onStop()`);
    onStop();
});
addEventListener("yt-navigate-finish", e =>{
    console.log(`${tag} yt-navigate-finish event fired, calling setup()`);
    setup()
});
addEventListener("beforeunload", e => {
    console.log(`${tag} beforeunload event fired, calling onStop()`);
    onStop();
});
