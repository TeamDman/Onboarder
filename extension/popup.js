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
        console.log("[Onboarder] Notes area modified. Current value:", this.value);

        // Build the note ID from the v= slug + the title of the video
        const id = getNoteId();

        // Create a POST request to the Rust HTTP server
        fetch('https://127.0.0.1:3000/set_note', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ 
                id: id,
                content: this.value
            }),
        })
        .then(response => response.text())
        .then(data => {
            console.log('[Onboarder] Success:', data);
        })
        .catch((error) => {
            console.error('[Onboarder] Error:', error);
        });
    });

    // Insert the text area after the video player element
    videoPlayerElement.insertAdjacentElement('afterend', textArea);
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

            console.log("[Onboarder] getting existing note content");
            let content = "";
            {
                const resp = await fetch(`https://127.0.0.1:3000/get_note?id=${getNoteId()}`);
                const data = await resp.json();
                content = data.content;
                console.log("[Onboarder] existing note content length: ", content.length);
            }
            addTextArea(videoPlayerElement, content);
            break;
        }
        await sleep(1000);
    }
}
main();