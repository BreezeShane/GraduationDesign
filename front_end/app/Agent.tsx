import axios from "axios";

function POST(url: string, data: any) {
axios.defaults.baseURL = `http://${process.env.BASE_URL}`;
    return axios.post(url, data, {
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        }
    })
}

function GET(url: string) {
axios.defaults.baseURL = `http://${process.env.BASE_URL}`;
    return axios.get(url, {
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        }
    })
}

export { POST, GET };