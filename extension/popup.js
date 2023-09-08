const onboarder_id = Math.random()

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
    const v = new URL(window.location).searchParams.get("v");
    const date = getLocalDate();
    const id = `[${date}] [youtube] [${v}] ${title.innerText}`;
    return id;
}

function getCurrentNoteContent() {
    return document.getElementById("custom_notes_area").value;
}

// Function to add the text area and attach an event listener to it
function addTextArea(videoArea, initialContent) {
    console.log("[Onboarder] adding content to page");

    // Unique ID for the text area
    const textAreaId = "custom_notes_area";

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
    const next = existing + content;
    await save(next);
    const note = document.getElementById("custom_notes_area");
    note.value = next;
}

function getVideoTimestamp() {
    const video = document.getElementsByClassName("html5-main-video")[0];
    return video.currentTime;
}

function addChips(videoArea) {
    console.log("[Onboarder] building action chips");

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
                console.log("[Onboarder] Inserting timestamp");
                await appendContent(
                    `\nvideo current time ${getVideoTimestamp()} at ${new Date().toString()}`
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
    console.log("[Onboarder] video paused");
    await appendContent(
        `\nvideo paused ${getVideoTimestamp()} seconds in at ${new Date().toString()}`
    );
}

async function onPlaying() {
    if (window.onboarder_id != onboarder_id) return;
    console.log("[Onboarder] video playing");
    await appendContent(
        `\nvideo playing from ${getVideoTimestamp()} seconds in at ${new Date().toString()}`
    );
}

function attachPauseAndPlayListeners(videoArea) {
    console.log("[Onboarder] attaching pause and play listeners");
    video = videoArea.querySelector("video");

    video.addEventListener("pause", onPause);
    video.addEventListener("playing", onPlaying);

    // // Save the current pause and play methods only if they haven't been saved yet
    // video.onboarder_original_pause =
    //     video.onboarder_original_pause || video.pause;
    // video.onboarder_original_play =
    //     video.onboarder_original_play || video.play;

    // // Override with your own
    // video.pause = function () {
    //     onPause();
    //     this.onboarder_original_pause();
    // };

    // video.play = function () {
    //     onPlaying();
    //     this.onboarder_original_play();
    // };
}

function save(content) {
    // Build the note ID from the v= slug + the title of the video
    const id = getNoteId();
    console.log(`[Onboarder] Saving video with note id \`${id}\` to disk`);

    // Create a POST request to the Rust HTTP server
    return fetch("https://127.0.0.1:3000/set_note", {
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
            console.log("[Onboarder] Success:", data);
        })
        .catch((error) => {
            console.error("[Onboarder] Error:", error);
        });
}

async function downloadVideo() {
    console.log("[Onboarder] Downloading video");
    const resp = await fetch("https://127.0.0.1:3000/download", {
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

async function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function setup() {
    window.onboarder_id = onboarder_id;
    console.log("[Onboarder] waiting for server healthcheck to succeed");
    while (true) {
        try {
            const resp = await fetch("https://127.0.0.1:3000/healthcheck");
            if (resp.status == 200) {
                break;
            }
        } catch (ignored) {}
        console.log(
            "[Onboarder] server healthcheck failed, retrying after 1 second"
        );
        await sleep(1000);
    }
    console.log("[Onboarder] server is running");

    console.log("[Onboarder] waiting for video player element");
    while (true) {
        let videoArea = document.getElementById("full-bleed-container");
        if (videoArea) {
            console.log("[Onboarder] video player element found");

            {
                console.log("[Onboarder] getting existing note content");
                let content = "";
                {
                    const resp = await fetch(
                        `https://127.0.0.1:3000/get_note?id=${getNoteId()}`
                    );
                    const data = await resp.json();
                    content = data.content;
                    console.log(
                        "[Onboarder] existing note content length: ",
                        content.length
                    );
                }
                addTextArea(videoArea, content);
            }

            addChips(videoArea);
            attachPauseAndPlayListeners(videoArea);
            break;
        }
        await sleep(1000);
    }
}
setup();
// https://stackoverflow.com/a/34100952/11141271
addEventListener("yt-navigate-finish", setup);
