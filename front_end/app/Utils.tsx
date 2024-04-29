import axios from "axios";
import { jwtDecode } from "jwt-decode";
import { UserSessionInfo } from "./Types";

const saveSessionUserInfo = (props: UserSessionInfo) => {
    const { useremail } = props;
    if (useremail) {
        sessionStorage.setItem('useremail', useremail);
    }
};

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

export { setAuthToken, parseAuthToken, saveSessionUserInfo };