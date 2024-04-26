import { isString } from "antd/es/button";
import axios from "axios";
import { jwtDecode } from "jwt-decode";

const setAuthToken = (token: string | undefined) => {
    if(token){
        axios.defaults.headers.common['Authorization'] = token;
        sessionStorage.setItem('token', token);
    } else {
        delete axios.defaults.headers.common['Authorization'];
        sessionStorage.removeItem('token');
    }
}

const parseAuthToken = (token: string) => {
    if (token) {
        return jwtDecode(token);
    } else {
        return null;
    }
}

export { setAuthToken };