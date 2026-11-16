// One Apple Event per property for the whole address book, instead of one per person per
// field: bulk reads return every person's value in a single array, nested ones as arrays of arrays.
function text(raw) {
    if (raw === null || raw === undefined) return '';
    if (raw instanceof Date) return raw.toISOString();
    return String(raw);
}

function labelText(raw) {
    const match = /^_\$!<(.*)>!\$_$/.exec(text(raw));
    return match ? match[1] : text(raw);
}

function contacts() {
    const people = Application('Contacts').people;
    const scalars = [
        ['name', 'name', 'Name'], ['firstName', 'name', 'First name'],
        ['middleName', 'name', 'Middle name'], ['lastName', 'name', 'Last name'],
        ['nickname', 'name', 'Nickname'], ['maidenName', 'name', 'Maiden name'],
        ['title', 'name', 'Title'], ['suffix', 'name', 'Suffix'],
        ['phoneticFirstName', 'name', 'Phonetic first name'],
        ['phoneticMiddleName', 'name', 'Phonetic middle name'],
        ['phoneticLastName', 'name', 'Phonetic last name'],
        ['organization', 'work', 'Organization'], ['department', 'work', 'Department'],
        ['jobTitle', 'work', 'Job title'], ['homePage', 'link', 'Home page'],
        ['birthDate', 'other', 'Birthday'], ['note', 'other', 'Notes'],
        ['creationDate', 'other', 'Created'], ['modificationDate', 'other', 'Modified']
    ];
    const labeled = [
        ['phones', 'phone', 'Phone'], ['emails', 'email', 'Email'],
        ['urls', 'link', 'URL'], ['customDates', 'other', 'Date'],
        ['relatedNames', 'other', 'Related name']
    ];
    const ids = people.id();
    const columns = {};
    scalars.forEach(function (field) { columns[field[0]] = people[field[0]](); });
    const lists = {};
    labeled.forEach(function (field) {
        const items = people[field[0]];
        lists[field[0]] = {id: items.id(), label: items.label(), value: items.value()};
    });
    const addresses = {id: people.addresses.id(), label: people.addresses.label()};
    ['street', 'city', 'state', 'zip', 'country', 'countryCode'].forEach(function (key) {
        addresses[key] = people.addresses[key]();
    });
    const messaging = {
        id: people.instantMessages.id(), label: people.instantMessages.label(),
        serviceName: people.instantMessages.serviceName(), userName: people.instantMessages.userName()
    };
    const social = {
        id: people.socialProfiles.id(), serviceName: people.socialProfiles.serviceName(),
        userName: people.socialProfiles.userName(), userIdentifier: people.socialProfiles.userIdentifier(),
        url: people.socialProfiles.url()
    };
    const rows = [];
    for (let i = 0; i < ids.length; i++) {
        const fields = [];
        function add(id, group, label, value) {
            if (value.trim()) fields.push({id: id, group: group, label: label, value: value});
        }
        scalars.forEach(function (field) { add(field[0], field[1], field[2], text(columns[field[0]][i])); });
        labeled.forEach(function (field) {
            const list = lists[field[0]];
            for (let j = 0; j < list.id[i].length; j++) {
                add(text(list.id[i][j]), field[1], labelText(list.label[i][j]) || field[2], text(list.value[i][j]));
            }
        });
        for (let j = 0; j < addresses.id[i].length; j++) {
            const address = ['street', 'city', 'state', 'zip', 'country', 'countryCode']
                .map(function (key) { return text(addresses[key][i][j]); }).filter(Boolean).join(', ');
            add(text(addresses.id[i][j]), 'address', labelText(addresses.label[i][j]) || 'Address', address);
        }
        for (let j = 0; j < messaging.id[i].length; j++) {
            add(text(messaging.id[i][j]), 'link', labelText(messaging.label[i][j]) || 'Instant message',
                [text(messaging.serviceName[i][j]), text(messaging.userName[i][j])].filter(Boolean).join(': '));
        }
        for (let j = 0; j < social.id[i].length; j++) {
            add(text(social.id[i][j]), 'link', text(social.serviceName[i][j]) || 'Social profile',
                [text(social.userName[i][j]), text(social.userIdentifier[i][j]), text(social.url[i][j])].filter(Boolean).join(' '));
        }
        rows.push({id: text(ids[i]), name: text(columns.name[i]), fields: fields});
    }
    return rows;
}
