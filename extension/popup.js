// Function to remove an existing element by its ID
function removeElementById(id) {
    let element = document.getElementById(id);
    if (element) {
        element.parentNode.removeChild(element);
    }
}

function getNoteId() {
    const title = document.querySelector("#title.ytd-watch-metadata");
    const v = new URL(window.location).searchParams.get("v");
    const date = new Date().toISOString().split('T')[0];
    const id = `[${date}] [youtube] [${v}] ${title.innerText}`
    return id;
}

function getCurrentNoteContent() {
    return document.getElementById("custom_notes_area").value;
}

// Function to add the text area and attach an event listener to it
function addTextArea(videoPlayerElement, initialContent) {
    console.log("[Onboarder] adding content to page");

    // Unique ID for the text area
    const textAreaId = 'custom_notes_area';

    // Remove the existing text area if it exists
    removeElementById(textAreaId);

    // Create the text area element
    let textArea = document.createElement('textarea');
    textArea.value = initialContent;

    // Assign a unique ID to the text area
    textArea.id = textAreaId;

    // Style the text area
    textArea.style.width = 'calc(100% - 35px)';
    textArea.style.padding = '10px';
    textArea.style.marginTop = '20px';
    textArea.style.marginLeft = '5px';
    textArea.style.borderRadius = '8px';
    textArea.style.backgroundColor = '#1f1f1f';
    textArea.style.color = '#ffffff';
    textArea.placeholder = 'notes';

    // Add a change event listener to send the content to the Rust endpoint
    textArea.addEventListener('input', function() {
        save(this.value);
    });

    // Insert the text area after the video player element
    videoPlayerElement.insertAdjacentElement('afterend', textArea);
}

function addChips(videoPlayerElement) {
    removeElementById('chip_container');
    const chipContainer = document.createElement('div');
    chipContainer.id = 'chip_container';
    chipContainer.style.display = 'flex';
    chipContainer.style.flexWrap = 'wrap';

    
    const actions = [
        {
            text: 'Timestamp',
            description: 'Insert the current video timestamp',
            action: async function() {

                console.log("[Onboarder] Inserting timestamp");
                const video = document.getElementsByClassName("html5-main-video")[0];
                const timestamp = video.currentTime;
                const content = getCurrentNoteContent() + `\nvideo current time ${video.currentTime} at ${new Date().toISOString()}`;
                await save(content);

                const note = document.getElementById("custom_notes_area");
                note.value = content;
            }
        },
        {
            text: 'Download',
            description: 'Save the video to disk',
            action: async function() {
                await downloadVideo();
            }
        },
    ];
    actions.forEach((action) => {
        const chip = document.createElement('button');
        chip.innerText = action.text;
        chip.title = action.description;
        chip.style.margin = '5px';
        chip.style.padding = '10px';
        chip.style.borderRadius = '12px';
        chip.style.cursor = 'pointer';
        chip.style.backgroundColor = '#1f1f1f';
        chip.style.color = '#ffffff';
        
        chip.addEventListener('click', action.action);

        chipContainer.appendChild(chip);
    });

    videoPlayerElement.insertAdjacentElement('afterend', chipContainer);
}

function save(content) {
    // Build the note ID from the v= slug + the title of the video
    const id = getNoteId();
    console.log(`[Onboarder] Saving video with note id \`${id}\` to disk`);


    // Create a POST request to the Rust HTTP server
    return fetch('https://127.0.0.1:3000/set_note', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
            id,
            content,
        }),
    })
    .then(response => response.text())
    .then(data => {
        console.log('[Onboarder] Success:', data);
    })
    .catch((error) => {
        console.error('[Onboarder] Error:', error);
    });
}

async function downloadVideo() {
    console.log("[Onboarder] Downloading video");
    const resp = await fetch('https://127.0.0.1:3000/download', {
        method: "POST",
        headers: {
            'Content-Type': 'application/text',
        },
        body: window.location.href + "",
    });
    if (resp.status == 200) {
        const fileId = await resp.text();
        const content = getCurrentNoteContent() + `\nDownloaded video, file id: ${fileId}\ end`;
        await save(content);
        const note = document.getElementById("custom_notes_area");
        note.value = content;
    } else {
        const content = getCurrentNoteContent() + `\nFailed to download video, status code: ${resp.status}`;
        const note = document.getElementById("custom_notes_area");
        note.value = content;
    }
}

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function main(){ 
    console.log("[Onboarder] waiting for server healthcheck to succeed");
    while(true){
        try {
            const resp = await fetch('https://127.0.0.1:3000/healthcheck');
            if (resp.status == 200) {
                break;
            }
        } catch (ignored) {
        }
        console.log("[Onboarder] server healthcheck failed, retrying after 1 second");
        await sleep(1000);
    }
    console.log("[Onboarder] server is running");

    
    console.log("[Onboarder] waiting for video player element");
    while (true) {
        let videoPlayerElement = document.getElementById('full-bleed-container');
        if (videoPlayerElement) {
            console.log("[Onboarder] video player element found");

            {
                console.log("[Onboarder] getting existing note content");
                let content = "";
                {
                    const resp = await fetch(`https://127.0.0.1:3000/get_note?id=${getNoteId()}`);
                    const data = await resp.json();
                    content = data.content;
                    console.log("[Onboarder] existing note content length: ", content.length);
                }
                addTextArea(videoPlayerElement, content);
            }

            {
                console.log("[Onboarder] building action chips");
                addChips(videoPlayerElement);

            }
            break;
        }
        await sleep(1000);
    }
}
main();