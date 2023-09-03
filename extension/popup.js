// Function to remove an existing element by its ID
function removeElementById(id) {
    let element = document.getElementById(id);
    if (element) {
        element.parentNode.removeChild(element);
    }
}

// Function to add the text area and attach an event listener to it
function addTextArea(videoPlayerElement) {
    console.log("[Onboarder] adding content to page");

    // Unique ID for the text area
    const textAreaId = 'custom_notes_area';

    // Remove the existing text area if it exists
    removeElementById(textAreaId);

    // Create the text area element
    let textArea = document.createElement('textarea');

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
        const title = document.querySelector("#title.ytd-watch-metadata");
        const v = new URL(window.location).searchParams.get("v");
        const date = new Date().toISOString().split('T')[0];
        const id = `[${date}] [youtube] [${v}] ${title.innerText}`

        // Create a POST request to the Rust HTTP server
        fetch('http://127.0.0.1:3000/set_note', {
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
            const resp = await fetch('http://127.0.0.1:3000/healthcheck');
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
            addTextArea(videoPlayerElement);
            break;
        }
        await sleep(1000);
    }
}
main();