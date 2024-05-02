import { Pagination, PaginationProps } from "antd";
import { useState, memo, useEffect } from "react";
import { text } from "stream/consumers";

const ResultPagePanel: React.FC<{ result_table: Object }> = memo((props) => {
    const { result_table } = props;
    const [results, setResults] = useState({});
    const [total, setTotal] = useState(0);

    const [current, setCurrent] = useState(0);
    const [pageSize, setPageSize] = useState(10);

    const [title, setTitle] = useState("");
    const [content, setContent] = useState("");
    const [link, generateLink] = useState("Null");

    const updatePanelbyIndex = (page_num: number, raw_table: Object) => {
        let idx = page_num - 1;
        let table = Object.entries(raw_table);
        if (table.length > 0){
            let related_image_name = (table[idx][1])[0]
            let specie_name = (table[idx][1])[1];
            generateLink(`${specie_name}`)
            setTitle(`${specie_name} - ${related_image_name}`);
            setContent(``); //the content from website searched for specie_name
        }
    };

    useEffect(() => {
        setResults(result_table);
        setTotal(Object.keys(result_table).length);
        setCurrent(0);
        updatePanelbyIndex(1, result_table);
    }, [result_table])

    const onChange: PaginationProps['onChange'] = (page) => {
        console.log(page);
        setCurrent(page);
        updatePanelbyIndex(page, results)
    };
    return (
        <div style={{ height: "80%" }}>
            <div style={{ height: "10%" }}>
                Click <a href={link}>{link}</a> For More Detalis.
            </div>

            <div style={{ height: "70%" }}>
                <h1>{title}</h1>
                <p>{content}</p>
            </div>
            <br />
            <Pagination
                total={total}
                showQuickJumper
                showTotal={(total, range) => `${range[0]} of ${total} items`}
                defaultCurrent={0}
                defaultPageSize={1}
                current={current}
                onChange={onChange}
            />
        </div>
    );
})

export default ResultPagePanel;