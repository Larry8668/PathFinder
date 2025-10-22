import { Link } from "react-router-dom"
import snapshort1 from '../assets/snapshot1.png'
import snapshot2 from '../assets/snapshort2.png'


function ClipboardGuide(){
    return(
        <div className="flex flex-col justify-center gap-5 p-5 h-full bg-[#3D3C3C] font-sans text-white">
            <div className="felx flex-row justify-left text-left text-4xl ">Clipboard:</div>
            <div className="text-left backdrop-blur-sm">
                A clipboard manager feature that displays all copied items with details like timestamp, size, and usage count. Users can easily view, search, and re-copy any saved entry, making it simple to manage frequently used texts or data efficiently.
            </div>
            <div className="m-5 px-10 pt-10 rounded-t-xl bg-[#929292]">
                <img src={snapshort1} alt='snapshot1'
                className="flex justify-center items-center rounded-t-xl "/>
            </div>

            <div className="text-left mt-5 backdrop-blur-sm">
                A clipboard search feature that lets users quickly find any saved clipboard entry by typing keywords. It filters items in real-time, showing relevant results with details like timestamp, size, and usage, enabling fast retrieval and efficient clipboard management.                </div>
            <div className="m-5 px-10 pt-10 rounded-t-xl bg-[#929292]">
                <img src={snapshot2} alt='snapshot2'
                className="flex justify-center items-center rounded-t-xl "/>
            </div>

            <Link to='/OnlineSearchGuide'><button className="relative bottom-3 left-170 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button></Link>

        </div>
    )
}
export default ClipboardGuide