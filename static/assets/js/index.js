let has_sent_results = false;

let disable_submit = false;

let scrutins_list = {};
let acteurs_list = {};
let user_opinion = [];
let results = {};
let question_count = -1;
let needsTableRefresh = 1;

window.onload = async function() {

    /* load user opinion and participation state from local storage */
    if (localStorage.getItem("has_sent_results")) {
        has_sent_results = (await localStorage.getItem("has_sent_results") == "true");
    }
    if (localStorage.getItem("user_opinion")) {
        user_opinion = await JSON.parse(localStorage.getItem("user_opinion"));

        /* hide begin test button, display show results button instead */
        get("begin-test-intro").classList.add("hidden");
        get("begin-test-intro-results").classList.remove("hidden");
    }

    /* first load all scrutins.
     * blocking task to avoid race condition */
    await fetchScrutins();
    
    await fetchCsrfToken();

    /* "Skip test" button */
    fadeOut(get("skip-intro"), get("intro"), "fade-out-right", function() {
        loadScrutinsTable();

        /* force load results in background for the "edit answers" case */
        if (scrutins_list.length == user_opinion.length) {
            question_count = scrutins_list.length;
            get("intro-welcome-text").classList.add("hidden");
            get("test-results").classList.remove("hidden");
            loadResults();
            get("sidebar-edit-answers").classList.remove("disabled");
        }

        fadeIn(null, get("choices-main-table"), "fade-in-left", null);
    });
    /* to go faster */
    /*get("intro").classList.add("hidden");
    get("choices-main-table").classList.remove("hidden");*/

    /* "begin test" button */
    fadeOut(get("begin-test-intro"), get("intro-welcome-text"), "fade-out-right", function() {
        /* load question before fading in */
        let menuType = askQuestion(++question_count);
        fadeIn(null, get(menuType), "fade-in-left", null);
    });

    fadeOut(get("begin-test-intro-results"), get("intro-welcome-text"), "fade-out-right", function() {
        question_count = scrutins_list.length;
        fadeIn(null, get("test-results"), "fade-in-left", null);
        loadResults();
    });

    /* "why section" buttons and interface */
    fadeIn(get("sidebar-why"), get("why-view"), "fade-in-left", null);
    fadeOut(get("why-back-top"), get("why-view"), "fade-out-left", null);
    fadeOut(get("why-back-bottom"), get("why-view"), "fade-out-left", null);

    /* sidebar matches toggle */
    
    get("sidebar-toggle-matches").addEventListener('click', function () {
        if (get("matches-box").classList.contains("hidden")) {
            fadeIn(null, get("matches-box"), "fade-in-up", function () {
                get("sidebar-toggle-matches").innerHTML = "Masquer les correspondances";

            });
        }
        else {
            fadeOut(null, get("matches-box"), "fade-out-up", function () {
                get("sidebar-toggle-matches").innerHTML = "Afficher les correspondances";

            });
        }
    });

    /* "Sources" button in test */
    fadeOut(get("test-sources"), get("question-box"), "fade-out-up", function() {
        fadeIn(null, get("sources-view"), "fade-in-down", null);
    });

    fadeOut(get("sources-back"), get("sources-view"), "fade-out-down", function() {
        fadeIn(null, get("question-box"), "fade-in-up", null);
    });

    /* "pour", "abstention", "contre" buttons */
    fadeOut(get("test-vote-for"), get("question-box"), "fade-out-right", function() {
        userVote("pour");

        let menuType = askQuestion(++question_count);
        fadeIn(null, get(menuType), "fade-in-left", null);
        if (menuType == "test-results") loadResults();
    });

    fadeOut(get("test-vote-against"), get("question-box"), "fade-out-right", function() {
        userVote("contre");
        
        let menuType = askQuestion(++question_count);
        fadeIn(null, get(menuType), "fade-in-left", null);
        if (menuType == "test-results") loadResults();
    });

    fadeOut(get("test-vote-not"), get("question-box"), "fade-out-right", function() {
        userVote("abstention");
        
        let menuType = askQuestion(++question_count);
        fadeIn(null, get(menuType), "fade-in-left", null);
        if (menuType == "test-results") loadResults();
    });

    /* back button */
    fadeOut(get("test-back"), get("question-box"), "fade-out-left", function() {
        let menuType = askQuestion(--question_count);
        fadeIn(null, get(menuType), "fade-in-right", null);
    });
    
    /* back button on results screen */
    fadeOut(get("test-back-results"), get("test-results"), "fade-out-left", function() {
        let menuType = askQuestion(--question_count);
        fadeIn(null, get(menuType), "fade-in-right", null);
        get("match-groupe").classList.add("hidden");
        get("match-depute").classList.add("hidden");
        get("match-view").classList.add("hidden");
        get("results-graph").classList.add("hidden");
        get("results-submit-div").classList.add("hidden");
        get("results-scroll-arrow").classList.add("hidden");
        get("results-details").classList.add("hidden");
        get("test-back-results").classList.add("disabled");
        get("test-reset").classList.add("disabled");
        get("test-view-details").classList.add("disabled");
    });

    /* send results button, with a ninja Thanks on click */
    get("results-send").addEventListener('click', function () {
        get("results-send").innerHTML = "Merci !";
    });
    fadeOut(get("results-send"), get("results-submit-div"), "fade-out-right", function() {
        fetchSubmit();
        
        has_sent_results = true;
        localStorage.setItem("has_sent_results", has_sent_results.toString());
    });

    /* details button on results screen */
    fadeOut(get("results-details"), get("intro"), "fade-out-right", viewDetails);
    fadeOut(get("test-view-details"), get("intro"), "fade-out-right", viewDetails);

    /* return to begin test screen if test isn't complete */
    fadeOut(get("begin-test-from-table"), get("choices-main-table"), "fade-out-right", function() {
        fadeIn(null, get("intro"), "fade-in-left", null);
    });

    /* return to results with edit answers button */
    fadeOut(get("sidebar-edit-answers"), get("choices-main-table"), "fade-out-right", function() {
        get("sidebar-edit-answers").classList.add("disabled");
        fadeIn(null, get("intro"), "fade-in-left", null);
    });

    /* completely reset the application */
    fadeOut(get("test-reset"), get("intro"), "fade-out-right", function() {
        localStorage.clear();
        location.reload();
    });
}

function viewDetails() {
    loadScrutinsTable();
    get("sidebar-edit-answers").classList.remove("disabled");
    fadeIn(null, get("choices-main-table"), "fade-in-left", null);
}

async function fetchScrutins() {
    await fetch(`/data/scrutins.json`)
        .then(function (response) {
            return response.json();
        })
        .then(function(data) {
            scrutins_list = data;
        })
        .catch(function (err) {
            console.log(err);
            alert("Le chargement des données a échoué. Veuillez actualiser la page.");
        });

    await fetch(`/data/acteurs.json`)
        .then(function (response) {
            return response.json();
        })
        .then(function(data) {
            acteurs_list = data;
        })
        .catch(function (err) {
            console.log(err);
            alert("Le chargement des données a échoué. Veuillez actualiser la page.");
        });

    await refreshSidebar();
    //await loadScrutinsTable();
    console.log("Ready.");
}

async function fetchCsrfToken() {
    await fetch(`/api/csrftoken`)
    .then(function (response) {
        return response.json();
    })
    .then(function(data) {
        localStorage.setItem("csrf_token", data.csrf_token);
    })
    .catch(function (err) {
        console.log(err);
        // silently disable submission if we can't get the CSRF token
        disable_submit = true;
    });
    console.log("Got CSRF token from API.");
}

async function fetchResults() {
    await fetch(`/api/results`)
    .then(function (response) {
        return response.json();
    })
    .then(function(data) {
        results = data;
    })
    .catch(function (err) {
        console.log(err);
        // silently skip results if we can't get them
    });
    console.log("Got results from API.");
}

async function fetchSubmit() {
    let csrf = 
    fetch("/api/submit", {
        method: "POST",
        body: JSON.stringify({
            csrf_token: await localStorage.getItem("csrf_token"),
            results: user_opinion,
        }),
        headers: {
            "Content-type": "application/json; charset=UTF-8"
        }
    })
        .then(response => response.text())
        .then(text => console.log(`Server returned ${text}.`));
}

async function refreshSidebar() {
    /* initialize user opinion */
    /* opinion.deputes = all deputees with at least 1 matching vote */
    /* opinion.organes = all organes */

    /* List all groups in the sidebar */
    get("choices-groupes").innerHTML = generateOrganesList();
    get("choices-groupes").scrollTo(0, 0);
    fadeIn(null, get("choices-groupes"), "fade-in-right", null);

    /* List the top20 deputees in the sidebar */
    get("choices-deputes").innerHTML = generateDeputeesList();
    get("choices-deputes").scrollTo(0, 0);
    fadeIn(null, get("choices-deputes"), "fade-in-right", null);

    get("skip-intro").classList.remove("disabled");
    get("begin-test-intro").classList.remove("disabled");
    get("begin-test-intro-results").classList.remove("disabled");
    get("sidebar-toggle-matches").classList.remove("disabled");
}

/* save user vote ("pour", "contre" or "abstention")
 * in user_opinion array */
function userVote(vote) {

    let scrutin_id = scrutins_list[question_count].question_id;
    let user_index = user_opinion.findIndex(u => u.question_id == scrutin_id);
    /* check if the user already voted for the same scrutin */
    if (user_index === -1) {
        let new_vote = {};
        new_vote["question_id"] = scrutin_id;
        new_vote["vote"] = vote;
        user_opinion.push(new_vote);
    }
    else {
        /* if they already voted, update the vote */
        user_opinion[user_index].vote = vote;
    }

    needsTableRefresh = 1;
    /* refresh results */
    refreshSidebar();
}

function appendVote(actorid, score, votes_array) {
    let actor_index = votes_array.findIndex(a => a.id == actorid);
    if (actor_index === -1) {
        // if the actor doesn't exist in list, add it
        let new_actor = {};
        new_actor["id"] = actorid;
        new_actor["score"] = score;
        votes_array.push(new_actor);
    }
    else {
        // else, take the existing actor and edit its score
        votes_array[actor_index].score += score;
    }
    return votes_array;
}

/* calculate organes / deputees score */
/* actorType must be either "organes" or "deputes" */
function calculateScore(actorType) {

    /* usersorted_actors is [ { id(actorType), score }, ...] */
    let usersorted_actors = [];

    /* user_opinion is [{ id(scrutin), vote}, ...] */


    for (let i = 0; i < user_opinion.length; i++) {
        /* user_opinion[i].id is question_id */
        /* user_opinion[i].vote is "pour", "contre" or "abstention" */
        let scrutin = scrutins_list.find(scr => scr.question_id == user_opinion[i].question_id);

        let default_score = (scrutin.invert_votes ? -1 : 1);

        /* if the actor's vote matches with the user's vote: +1 */
        for (let j = 0; j < scrutin[actorType][user_opinion[i].vote].length; j++) {
            let score = (user_opinion[i].vote == "abstention" ? 1 : default_score);

            usersorted_actors = appendVote(scrutin[actorType][user_opinion[i].vote][j], score, usersorted_actors);
        }

        /* if the actor's vote is *opposed* to the user's vote: -1 */
        /* this applies only for "pour" and "contre", not "abstention" */
        if (user_opinion[i].vote == "pour" || user_opinion[i].vote == "contre") {
            let opposite_vote = (user_opinion[i].vote == "pour" ? "contre" : "pour");
            let score = -default_score;

            for (let j = 0; j < scrutin[actorType][opposite_vote].length; j++) {
                usersorted_actors = appendVote(scrutin[actorType][opposite_vote][j], score, usersorted_actors);
            }
        }
    }

    /* append missing actors (because of abstention) */
    for (let i = 0; i < acteurs_list[actorType].length; i++) {
        usersorted_actors = appendVote(acteurs_list[actorType][i].id, 0, usersorted_actors);
    }

    /* convert usersorted_actors to [object, vote] */
    let usersorted_objects = [];

    if (actorType == "organes") {
        usersorted_actors.map((actor, index) => {
            let new_actor = {};
            new_actor["data"] = acteurs_list.organes.find(o => o.id == actor.id);
            new_actor["score"] = actor.score;
            usersorted_objects.push(new_actor);
        });
    }
    else {
        usersorted_actors.map((actor, index) => {
            let new_actor = {};
            new_actor["data"] = acteurs_list.deputes.find(d => d.id == actor.id);
            new_actor["score"] = actor.score;
            usersorted_objects.push(new_actor);
        });
    }

    /* sort the array, best score first */
    usersorted_objects.sort(function (a, b) {
        return a.score - b.score;
    }).reverse();

    return usersorted_objects;
}

/* template-related functions */

function generateOrganesList() {
    /* calculate organes score based on user opinion */

    /* then display it in sidebar */
    /* do not display non-inscrits as a group (organes.display === false) */
    if (user_opinion.length == 0) {
        return `
        ${acteurs_list.organes.filter(org => org.display).map((organe, index) =>
            `<div class="depute" data-id="${organe.id}">
                <div>
                    <figure class="ball" style="background-color: ${fmt(organe.color)}"></figure>
                    <span class="name">${organe.name}</span>
                </div>
                <span class="mark">0&nbsp;/&nbsp;${scrutins_list.length}</span>
            </div>`
        ).join("")}
        `;
    }
    else {
        let organes_score = calculateScore("organes");
        return `
    ${organes_score.map((organe, index) =>
        `<div class="groupe${organe.data.display ? `` : ` hidden`}" data-id="${organe.data.id}">
            <div>
                <figure class="ball" style="background-color: ${fmt(organe.data.color)}"></figure>
                <span class="name">${organe.data.name}</span>
            </div>
            <span class="mark" data-mark="${organe.score}">${organe.score}&nbsp;/&nbsp;${scrutins_list.length}</span>
         </div>`
    ).join("")}
    `;
    }
}

function generateDeputeesList() {
    if (user_opinion.length == 0) {
        /* return the 20 first deputees with a score of zero */
        return `
        ${acteurs_list.deputes.slice(0, 20).map((dep, index) =>
            `<div class="depute" data-id="${dep.id}">
                <div>
                    <figure class="ball" style="background-color: ${fmt(acteurs_list.organes.find(o => o.id == dep.organe).color)}"></figure>
                    <span class="name">${dep.name}</span>
                </div>
                <span class="mark">0&nbsp;/&nbsp;${scrutins_list.length}</span>
            </div>`
        ).join("")}
        `;
    }
    else {
        let deputes_score = calculateScore("deputes");
        return `
        ${deputes_score.slice(0, 20).map((dep, index) =>
            `<div class="depute" data-id="${dep.data.id}">
                <div>
                    <figure class="ball" style="background-color: ${fmt(acteurs_list.organes.find(o => o.id == dep.data.organe).color)}"></figure>
                    <span class="name">${dep.data.name}</span>
                </div>
                <span class="mark">${dep.score}&nbsp;/&nbsp;${scrutins_list.length}</span>
            </div>`
        ).join("")}
        `;
    }
}

/* scrutins-table: insert one <tr> for each deputee */

function generateScrutinsTable() {
    let score_deputes = calculateScore("deputes");
    let score_organes = calculateScore("organes");
    return `
    <thead>
        <tr id="table-header-row">
            <th>Nom</th>
            ${scrutins_list.map((scrutin_data, index) =>
                `<th>${index+1}</th>`
            ).join("")}
        </tr>
    </thead>
    <tbody>
        <tr class="table-user separator ${user_opinion.length != scrutins_list.length ? ` hidden` : ``}" data-type="user">
            <td data-colid="-1" class="match">Votre position</td>
        ${user_opinion.map((user, index_y) =>
            `<td data-colid="${index_y}" class="match">${user.vote.replace("abstention", "abst.")}</td>`
        ).join("")}
        </tr>
        <tr data-type="user">
            <td data-colid="-1" colspan="${scrutins_list.length + 1}" class="group-header">Groupes politiques</td>
        </tr>
        ${score_organes.filter(org => org.data.display).map((organe, index) =>
            `<tr data-id="${organe.data.id}" data-type="organe" class="table-organe">
                <td data-colid="-1">
                <figure class="ball" style="background-color: ${fmt(organe.data.color)}"></figure>
                ${fmt(organe.data.name)}
                </td>
                ${scrutins_list.map((scrutin_data, index_y) =>
                    `<td data-colid="${index_y}" data-rowid="${index}">${checkVoteOrgane(organe.data.id, scrutin_data)}</td>`
                ).join("")}
            </tr>`
        ).join("")}
        <tr data-type="user">
            <td data-colid="-1" colspan="${scrutins_list.length + 1}" class="group-header">Députés</td>
        </tr>
        ${score_deputes.map((acteur, index) =>
            `<tr data-id="${acteur.data.id}" data-type="depute">
                <td data-colid="-1">
                <figure class="ball" style="background-color: ${fmt(acteurs_list.organes.find(o => o.id == acteur.data.organe).color)}"></figure>
                ${fmt(acteur.data.name)}
                </td>
                ${scrutins_list.map((scrutin_data, index_y) =>
                    `<td data-colid="${index_y}" data-rowid="${index}">${checkVote(acteur.data.id, scrutin_data)}</td>`
                ).join("")}
             </tr>`
        ).join("")}
    </tbody>
`};

function addScrutinsTableEvents() {
    /* add mouseover to display vote description */
    let table_cells = get("scrutins-table").getElementsByTagName("td");
    for (let i = 0; i < table_cells.length; i++) {
        table_cells[i].addEventListener('mouseover', function() {
            table_mouseover(table_cells[i].parentElement.dataset, table_cells[i].dataset, table_cells[i].innerHTML);
        });
        table_cells[i].addEventListener('mouseout', function() {
            table_mouseout()
        });
    }

    /* add mouseover to get scrutin description, offset at 1 */
    let table_headers = get("table-header-row").children;
    for (let i = 1; i < table_headers.length; i++) {
        table_headers[i].addEventListener('mouseover', function() {
            headers_mouseover(i - 1);
        });
        table_headers[i].addEventListener('mouseout', function() {
            table_mouseout()
        });
    }
}

function addScrutinsTableMatches() {
    let table_rows = get("scrutins-table").rows;
    /* offset i and j to match the table data */
    for (let i = 2; i < table_rows.length; i++) {
        for (let j = 1; j < table_rows[i].children.length; j++) {
            if (user_opinion.length >= j) {
                if (table_rows[i].children[j].innerHTML == user_opinion[j - 1].vote.replace("abstention", "abst.")) {
                    table_rows[i].children[j].classList.add("match");
                }
                else if ((table_rows[i].children[j].innerHTML == "contre" && user_opinion[j - 1].vote == "pour") || (table_rows[i].children[j].innerHTML == "pour" && user_opinion[j - 1].vote == "contre")) {
                    table_rows[i].children[j].classList.add("unmatch");
                }
            }
        }
    }
}

function loadScrutinsTable() {
    console.log("Scrutins table checked");
    if (needsTableRefresh == 1) {
        console.log("Scrutins table refreshed");
        get("scrutins-table").innerHTML = generateScrutinsTable();
        addScrutinsTableEvents();
        addScrutinsTableMatches();
        if (user_opinion.length == scrutins_list.length) {
            get("begin-test-banner").classList.add("hidden");
        }
    }
    needsTableRefresh = 0;
}

function table_mouseover(cell_parent_data, cell_data, vote) {
    /* do nothing for the first column */
    if (cell_data.colid < 0) {
        return;
    }

    let desc_sentence = "";

    if (cell_parent_data.type == "organe") {
        desc_sentence += `Le groupe ${fmt(acteurs_list.organes.find(o => o.id == cell_parent_data.id).name)} `;
    }
    else if (cell_parent_data.type == "depute") {
        let depute_data = acteurs_list.deputes.find(d => d.id == cell_parent_data.id);
        desc_sentence += `${fmt(depute_data.name)} (${fmt(acteurs_list.organes.find(o => o.id == depute_data.organe).abrev)}) `;
    }
    else {
        desc_sentence += `Vous `;
    }

    if (vote == "abst.") {
        if (cell_parent_data.type == "user") {
            desc_sentence += `avez <u>choisi de vous abstenir</u> sur `;
        }
        else {
            desc_sentence += `a <u>choisi de s'abstenir</u> sur `;
        }
    }
    else if (vote == "N/A") {
        desc_sentence += `<u>n'était pas dans l'hémicycle</u> lors du vote sur `;
    }
    else {
        if (cell_parent_data.type == "user") {
            desc_sentence += `avez <u>voté ${fmt(vote)}</u> `;
        }
        else {
            desc_sentence += `a <u>voté ${fmt(vote)}</u> `;
        }
    }
    desc_sentence += `${scrutins_list[cell_data.colid].abbr}`;

    get("table-desc-content").innerHTML = desc_sentence;
}

function table_mouseout() {
    get("table-desc-content").innerHTML = "";
}

function headers_mouseover(i) {
    get("table-desc-content").innerHTML = scrutins_list[i].name;
}

function checkVote(acteur_paid, scrutin) {
    if (scrutin.deputes.pour.includes(acteur_paid)) {
        return (scrutin.invert_votes ? "contre" : "pour");
    }
    else if (scrutin.deputes.contre.includes(acteur_paid)) {
        return (scrutin.invert_votes ? "pour" : "contre");
    }
    else if (scrutin.deputes.abstention.includes(acteur_paid)) {
        return "abst.";
    }
    else {
        return "N/A";
    }
}

function checkVoteOrgane(orgid, scrutin) {
    if (scrutin.organes.pour.includes(orgid)) {
        return (scrutin.invert_votes ? "contre" : "pour");
    }
    else if (scrutin.organes.contre.includes(orgid)) {
        return (scrutin.invert_votes ? "pour" : "contre");
    }
    else if (scrutin.organes.abstention.includes(orgid)) {
        return "abst.";
    }
    else {
        return "N/A";
    }
}

/* generic functions */

function get(elem) {
    return document.getElementById(elem);
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

/* Fade-in and fade-outs
 * buttonEvent: for onclick->fade events
 * elem: the element to fade-in / fade-out
 * fadeType: the CSS property
 * fnTrigger: the function to trigger after fading in/out
 */

function fadeIn(buttonEvent, elem, fadeType, fnTrigger) {
    if (buttonEvent !== null) {
        // add event and recurse
        buttonEvent.addEventListener('click', async function a_click_fdin() {
            /* prevent ninja doubleclicks */
            buttonEvent.classList.add("disabled-silent");
            /* check if the element is already visible */
            if (elem.classList.contains("hidden")) {
                fadeIn(null, elem, fadeType, fnTrigger);
            }
            await sleep(1200);
            buttonEvent.classList.remove("disabled-silent");
        });
    }
    else {
        elem.addEventListener('animationend', function a_animend_fdin() {
            if (elem.classList.contains(fadeType)) {
                elem.classList.remove(fadeType);
            }
            elem.removeEventListener('animationend', a_animend_fdin);
        });
        elem.classList.add(fadeType);
        if (elem.classList.contains("hidden")) {
            elem.classList.remove("hidden");
        }
        if (fnTrigger != null) {
            fnTrigger();
        }
    }
}

function fadeOut(buttonEvent, elem, fadeType, fnTrigger) {
    if (buttonEvent !== null) {
        // add event and recurse
        buttonEvent.addEventListener('click', async function a_click_fdout() {
            buttonEvent.classList.add("disabled-silent");
            /* check if the element is hidden */
            if (!elem.classList.contains("hidden")) {
                fadeOut(null, elem, fadeType, fnTrigger);
            }
            await sleep(1200);
            buttonEvent.classList.remove("disabled-silent");
        });
    }
    else {
        elem.addEventListener('animationend', function a_animend_fdout() {
            if (!elem.classList.contains("hidden")) {
                elem.classList.add("hidden");
            }
            if (elem.classList.contains(fadeType)) {
                elem.classList.remove(fadeType);
            }
            if (fnTrigger != null) {
                fnTrigger();
            }
            elem.removeEventListener('animationend', a_animend_fdout);
        });
        elem.classList.add(fadeType);
    }
}

/* handling test */
/* returns "question-box" for test questions
 * returns "test-results" for test results
 * returns "intro-welcome-text" for home page */
function askQuestion(qcount) {
    let progress_pct = ~~((qcount / scrutins_list.length) * 10000) / 100;
    get("test-progress-top").children[0].style.width = `${progress_pct}%`;
    get("test-progress-bottom").children[0].style.width = `${progress_pct}%`;

    if (qcount < 0 && question_count < 0) {
        question_count = -1;
        return "intro-welcome-text";
    }
    if (qcount >= scrutins_list.length && question_count >= scrutins_list.length) {
        question_count = scrutins_list.length;
        return "test-results";
    }

    get("scrutin_name").innerHTML = fmt(scrutins_list[qcount].name);
    get("scrutin_description").innerHTML = fmt(scrutins_list[qcount].description);
    get("scrutin_arguments").innerHTML = gen_arguments(scrutins_list[qcount].arguments);
    get("scrutin_question").innerHTML = fmt(scrutins_list[qcount].question);
    get("scrutin_sources").innerHTML = gen_sources(scrutins_list[qcount].sources);
    get("scrutin_meta").innerHTML = `Question n°${qcount+1}, scrutin n°${scrutins_list[qcount].id.split('V').pop()}, voté le ${fmt(scrutins_list[qcount].dateScrutin)}, ${Math.round((scrutins_list[qcount].nbreVotants * 100) / 577)}&nbsp;% de participation`;


    return "question-box";
}

function argument_trtype(argtype) {
    if (argtype == "pour") {
        return "Argument pour";
    }
    else if (argtype == "contre") {
        return "Argument contre";
    }
    else {
        return `Argument ${argtype}`;
    }
}

function gen_arguments(arg_array) {
    let finalstr = "";
    arg_array.forEach((argument, idx) => {
        let tpl = `
        <div class="test-argument ${argument.type}">
            <p>${fmt(argument.comment)}</p>
            <span>${fmt(argument_trtype(argument.type))}</span>
        </div>
        `;
        finalstr += tpl;
    });
    return finalstr;
}

function gen_sources(src_array) {
    let finalstr = "<ul>";
    src_array.forEach((src, idx) => {
        finalstr += `<li><a href="${src.link}">${fmt(src.name)}</a></li>`;
    });
    finalstr += "</ul>";
    return finalstr;
}

async function loadResults() {
    /* we assume we have the complete set of user opinions at this point */
    /* warning: can trigger a race condition with userVote here */
    await sleep(500);

    localStorage.setItem("user_opinion", JSON.stringify(user_opinion));

    /* get info about matching group */
    let match_groupe;

    /* skip NI */
    for (let i = 0; i < get("choices-groupes").children.length ; i++) {
        if (get("choices-groupes").children[i].classList.contains("hidden")) {
            continue;
        }
        match_groupe = get("choices-groupes").children[i].dataset.id;
        break;
    }

    let organe_data_index = acteurs_list.organes.findIndex(d => d.id == match_groupe);
    let organe_data = acteurs_list.organes[organe_data_index];
    get("match-organe-img").src = `/assets/img/groupes/${organe_data.abrev}.png`;
    get("match-organe-name").innerHTML = `${organe_data.name}`;


    /* get info about matching depute */
    let match_depute = get("choices-deputes").children[0].dataset.id;
    let depute_data_index = acteurs_list.deputes.findIndex(d => d.id == match_depute);
    let depute_data = acteurs_list.deputes[depute_data_index];

    get("match-depute-img").src = `/assets/img/deputes/${depute_data.id.replace("PA", "")}.jpg`;
    let depute_groupe = acteurs_list.organes[acteurs_list.organes.findIndex(o => o.id == depute_data.organe)];
    get("match-depute-name").innerHTML = `${depute_data.name} (${depute_groupe.abrev})`;

    /* getting results from API */
    if (Object.keys(results).length == 0) {
        await fetchResults();
    }

    displayResults();

    await sleep(200);
    fadeIn(null, get("match-groupe"), "fade-in-left", null);
    await sleep(600);
    fadeIn(null, get("match-depute"), "fade-in-left", null);
    await sleep(600);
    fadeIn(null, get("match-view"), "fade-in-left", null);

    /* get and display the (holy?) graph */
    await sleep(600);
    
    /* do not show global results if we couldn't get them */
    if (Object.keys(results).length != 0) {
        fadeIn(null, get("results-graph"), "fade-in-left", null);
        await sleep(500);
    }

    /* if the user has already sent their data, don't display the send card */
    if (!disable_submit && !has_sent_results) {
        fadeIn(null, get("results-submit-div"), "fade-in-up", null);
        enableDataTooltip();
        await sleep(50);
    }
    
    /* add scroll arrow and condition to make it disappear */
    fadeIn(null, get("results-scroll-arrow"), "fade-in-left", null);
    get("test-results").addEventListener("scroll", function e_checkScroll(e) {
        if (((e.target.scrollTop * 100) / e.target.scrollTopMax) >= 90) {
            get("results-scroll-arrow").classList.add("hidden");
            get("test-results").removeEventListener("scroll", e_checkScroll);
        }
    });
    await sleep(100);

    fadeIn(null, get("results-details"), "fade-in-left", null);

    await sleep(100);

    get("test-back-results").classList.remove("disabled");
    get("test-reset").classList.remove("disabled");
    get("test-view-details").classList.remove("disabled");

}

function enableDataTooltip() {
    let tooltip_button = get("data-tooltip-button");
    let tooltip_elem = get("data-tooltip");

    fadeIn(tooltip_button, tooltip_elem, "fade-in-up", null);
    fadeOut(tooltip_elem, tooltip_elem, "fade-out-down", null);
}

function displayResults() {

    // personal score display
    let results_perso_tpl = "";
    for (let i = 0; i < get("choices-groupes").children.length ; i++) {
        if (get("choices-groupes").children[i].classList.contains("hidden")) {
            continue;
        }
        let orgscore = parseFloat(get("choices-groupes").children[i].children[1].dataset.mark);
        // scale to 0 -> 100
        // instead of -scrutins_list.length -> +scrutins_list.length
        let orgscore_100 = ((orgscore + scrutins_list.length) / (scrutins_list.length * 2) * 100).toFixed(1);

        // transform 100.0% into 100% (less characters, better display)
        orgscore_100 = (orgscore_100 == 100 ? 100 : orgscore_100);

        let groupes_perso = {};
        groupes_perso["id"] = get("choices-groupes").children[i].dataset.id;
        groupes_perso["opinion_pct"] = orgscore_100;
        results_perso_tpl += displayResults_tpl(groupes_perso, "opinion_pct");
    }
    get("match-view-content").innerHTML = results_perso_tpl;

    if (Object.keys(results).length == 0) {
        console.log("Global results unavailable.");
        return ;
    }

    results.groupes.sort(function (a, b) {
        return a.opinion_pct - b.opinion_pct;
    }).reverse();

    // global score display
    let results_tpl = "";
    for (let i = 0; i < results.groupes.length; i++) {
        let display = acteurs_list.organes[acteurs_list.organes.findIndex(o => o.id == results.groupes[i].id)].display;
        if (display) {
            results_tpl += displayResults_tpl(results.groupes[i], "value_median");
        }
    }

    /* display another message if the stats are not ready */
    if (results.global.participations.valid < 500) {
        results_tpl += `
        <div class="chart-info">
            <span>Statistiques non disponibles</span>
            <span>Au moins 500 participations requises.</span>
            <span>Encore ~${500-results.global.participations.valid} participations !</span>
        </div>
        `;
    }
    else {
        let gendate = new Date(results.global.generated_at);
        results_tpl += `
        <div class="chart-info">
            <span>Scrutin par jugement majoritaire</span>
            <span>${results.global.participations.valid} / ${results.global.participations.total} participations</span>
            <span>Dernière actualisation le ${gendate.toLocaleDateString()}</span>
        </div>
        `;
    }

    get("results-graph-content").innerHTML = results_tpl;

}

function displayResults_tpl(groupe, field_name) {
    let groupe_data = acteurs_list.organes[acteurs_list.organes.findIndex(o => o.id == groupe.id)];
    // balance the result width to avoid overflowing in results graph
    let opinion_pct_b = groupe[field_name] - ( groupe[field_name] * 0.25 );

    return `
    <div class="chart-container flex">
        <label title="${groupe_data.name}">${groupe_data.abrev}</label>
        <li style="width: ${opinion_pct_b}%; background-color: ${groupe_data.color};"></li><span>${groupe[field_name]}&nbsp;%</span>
    </div>
    `;
}

// escapes HTML and inserts non-breaking spaces
function fmt(unsafe) {
    // HTML escape
    if (unsafe != null) {
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
                .replace(/'/g, "&#039;")
                    .replace(/ \!/g, "&nbsp;!")
                    .replace(/ \?/g, "&nbsp;?")
                    .replace(/« /g, "«&nbsp;")
                    .replace(/ »/g, "&nbsp;»");
                }
    else {
        return null;
    }
}

/* function from PolitiScales, MIT license */
function download_image() {
      let canvas = document.getElementById("generatedResults");
      let link = document.createElement("a");
      link.href = canvas.toDataURL();
      link.download = "QuelParti_"+new Date().toLocaleDateString()+".png";
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
}
