import { NotificationInstance } from "antd/es/notification/interface";
import axios from "axios";
import { useEffect, useState } from "react";

const iframeStyle: React.CSSProperties = {
    position: "absolute",
    height: "100%",
    width: "100%"
}

const WebSSH: React.FC<{ messageClient: NotificationInstance }> = (props) => {
    const { messageClient } = props;
    const [dest_ssh, setDestSSH] = useState("");
    let email = sessionStorage.getItem('useremail');

    useEffect(() => {
        if (!email) {
            messageClient.error({
                message: `Forbidden Operation!`,
                description: "You should sign in first!",
                placement: 'topLeft',
                duration: 2,
            });
        }
        axios.post(`/admin/authenticate_ssh/${email}`, {})
        .then((res) => {
            if (res.status == 200){
                messageClient.success({
                    message: `Start WebSSH Server Success!`,
                    description: "You could use the web-based terminal!",
                    placement: 'topLeft',
                    duration: 2,
                });
                setDestSSH(res.data);
            }
        }).catch((err) => {
            console.log(err);
            messageClient.error({
                message: `Failed to run WebSSH Server!`,
                description: "You might have no permission to use WebSSH Server!",
                placement: 'topLeft',
                duration: 2,
            });
        })
    })
    return (
        <iframe src={dest_ssh} style={iframeStyle}/>
    );
}

export default WebSSH;