const path = require('path')
const fs = require('fs')
const os = require('os')
const cluster = require('cluster')
const Deque = require('collections/deque')

const app = require('fastify')({ logger: false })
const { Pool, Client } = require('pg')

const pool = new Pool({
    user: 'mm',
    database: 'list_of_users',
    password: '.'
})

function walk(l, dir) {
    const files = fs.readdirSync(dir);
    files.forEach((file) => {
        let filepath = path.join(dir, file);
        
        const stats = fs.lstatSync(filepath);
        if (stats.isDirectory()) {
            walk(l, filepath);
        } else if (stats.isFile()) {
            l.push(filepath);
        }
    });
}

function file_list() {
    let l = new Deque 
    walk(l, '/my_tmp/mainline')
    return l.sort()
}

app.get('/file_list', (req, res) => {
    let l = file_list()

    let h = '<ul>' 
    l.forEach((file) => {
        h += "<li>" + file + "</li>\n"
    })
    h += "</ul>"

    res.header("Content-type", "text/html")
    res.send(h)
})

const FILE = fs.readFileSync('manpage', 'utf8')
app.get('/file', (req, res) => {
    res.header("Content-type", "text/html")
    res.send(FILE)
})

const query = {
    name: 'get_all_users',
    text: 'SELECT * from users',
}
app.get('/users_json', (req, res) => {
    pool
        .query(query)
        .then(db_res => {
            const json = JSON.stringify(db_res.rows)
            res.header("Content-type", "application/json")
            res.send(json)
        })
        .catch(e => console.error(e.stack))
})

app.get('/users_html', async (req, res) => {
    let resp = '<style> .normal {background-color: silver;} .highlight {background-color: grey;} </style><body><table>'
    i = 0

    const rows = await pool.query('SELECT * from users')

    rows.rows.forEach((row) => {
        if (i % 25 == 0) {
            resp += "<tr><th>UID</th><th>First Name</th><th>Last Name</th></tr>"
        }

        let type = ""
        if (i % 5 == 0) {
            type = 'highlight'
        } else {
            type = 'normal'
        }

        resp += "<tr class=\""
            +type+"\"><td>"
            +row.uid+"</td><td>"
            +row.first_name+"</td><td>"
            +row.last_name+"</td></tr>"
        i++
    })

    resp += '</table></body>'
    res.header("Content-type", "text/html")
    res.send(resp)
})

const args = process.argv;
const port = args[2]
const c_sz = os.cpus().length

const start = async () => {
    try {
        console.log("starting on port", port)
        await app.listen({ port: port, host: '0.0.0.0' })
    } catch (err) {
        console.log(err)
        process.exit(1)
    }
}

if (c_sz > 1) {
    if (cluster.isMaster) {
        for (let i=0; i < c_sz; i++) {
            cluster.fork()
        }

        cluster.on("exit", function(worker) {
            console.log("worker", worker.id, " has exited")
        })
    } else {
        start()
    }
} else {
    start()
}

