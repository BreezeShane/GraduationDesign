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
    const [useremail, setUserEmail] = useState<string | null>(null);

    useEffect(() => {
        setUserEmail(sessionStorage.getItem('useremail'));
    })
    if (useremail) {
        axios.post(`/admin/authenticate_ssh/${useremail}`, {})
        .then((res) => {
            if (res.status == 200){
                messageClient.success({
                    message: `Start WebSSH Server Success!`,
                    description: "You could use the web-based terminal!",
                    placement: 'topLeft',
                    duration: 2,
                });
                console.log(res.data);
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
        });
    }

    return (
        <iframe src={dest_ssh} style={iframeStyle}/>
    );
}

export default WebSSH;