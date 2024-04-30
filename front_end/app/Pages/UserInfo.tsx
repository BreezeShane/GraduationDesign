import { Card, Input, Upload } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import axios from 'axios';
import { useState } from 'react';
import { NotificationInstance } from 'antd/es/notification/interface';

const UserInfo: React.FC<{ messageClient: NotificationInstance }> = (props) => {
    const { messageClient } = props;
    let current_useremail = sessionStorage.getItem('useremail');
    if (!current_useremail){
        messageClient.error({
            message: `Forbidden Operation!`,
            description: "You should sign in first!",
            placement: 'topLeft',
            duration: 2,
        });
        return;
    }
    const [nick_name, setNickName] = useState("");
    const [email, setEmail] = useState("");
    const [contribution, setContribution] = useState("");
    const [role, setRole] = useState("");
    axios.post(`/user/info/${current_useremail}`, {})
    .then((res) => {
        let response = res.data;
        setNickName(response.nick_name);
        setEmail(response.email);
        setContribution(response.contribution);
        setRole(response.role);
    }).catch((err) => {
        console.log(err);
        messageClient.error({
            message: `Failed to fetch your user info!`,
            description: "You could try it again later!",
            placement: 'topLeft',
            duration: 2,
        });
    })
    return (
        <div style={{ width: "50%" }}>
            <div>
                <p>
                    Nick Name:
                </p>
                <Input value={nick_name} disabled />
            </div>
            <div>
                <p>
                    User Email:
                </p>
                <Input value={email} disabled />
            </div>
            <div>
                <p>
                    Contributions ( The count of feedback you have sent. ):
                </p>
                <Input value={contribution} disabled />
            </div>
            <div>
                <p>
                    Identity:
                </p>
                <Input value={role} disabled />
            </div>
        </div>
    );
}

export default UserInfo;